#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <cmath>
#include <string>
#include "IPlatform.h"
#include "IDeviceManager.h"
#include "ICPUEx.h"
#include "DeviceType.h"

// This C adapter links into the Rust executable. The proprietary AMD SDK
// headers and binaries stay on the user's machine and are never distributed.
namespace {
HMODULE platformModule = nullptr;
IPlatform* platform = nullptr;
ICPUEx* cpu = nullptr;
bool sdkInitialized = false;

std::wstring appendBin(std::wstring root) {
    while (!root.empty() && (root.back() == L'\\' || root.back() == L'/')) {
        root.pop_back();
    }
    if (root.empty()) {
        return {};
    }
    return root + L"\\bin";
}

std::wstring sdkBinDirectory() {
    wchar_t path[1024] = {};
    DWORD size = GetEnvironmentVariableW(L"AMDRMMONITORSDKPATH", path, 1024);
    if (size > 0 && size < 1024) {
        return appendBin(path);
    }

    HKEY key = nullptr;
    if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, L"SOFTWARE\\AMD\\RyzenMasterMonitoringSDK",
                      0, KEY_QUERY_VALUE, &key) == ERROR_SUCCESS) {
        DWORD type = 0;
        DWORD bytes = sizeof(path);
        LONG result = RegQueryValueExW(key, L"InstallationPath", nullptr, &type,
                                       reinterpret_cast<LPBYTE>(path), &bytes);
        RegCloseKey(key);
        if (result == ERROR_SUCCESS && (type == REG_SZ || type == REG_EXPAND_SZ)) {
            path[1023] = L'\0';
            if (path[0] != L'\0') {
                return appendBin(path);
            }
        }
    }

    return L"C:\\Program Files\\AMD\\RyzenMasterMonitoringSDK\\bin";
}
}

extern "C" {

int ak500_sdk_open() {
    if (sdkInitialized && cpu) {
        return 0;
    }

    const std::wstring bin = sdkBinDirectory();
    const std::wstring dllPath = bin + L"\\Platform.dll";
    if (!SetDllDirectoryW(bin.c_str())) {
        return -1;
    }
    platformModule = LoadLibraryW(dllPath.c_str());
    if (!platformModule) {
        SetDllDirectoryW(nullptr);
        return -2;
    }

    using GetPlatformFunc = IPlatform& (*)();
    const auto getPlatform = reinterpret_cast<GetPlatformFunc>(
        GetProcAddress(platformModule, "GetPlatform"));
    if (!getPlatform) {
        FreeLibrary(platformModule);
        platformModule = nullptr;
        SetDllDirectoryW(nullptr);
        return -3;
    }

    try {
        platform = &getPlatform();
        if (!platform->Init()) {
            platform = nullptr;
            FreeLibrary(platformModule);
            platformModule = nullptr;
            SetDllDirectoryW(nullptr);
            return -4;
        }
        sdkInitialized = true;
        cpu = static_cast<ICPUEx*>(
            platform->GetIDeviceManager().GetDevice(dtCPU, 0));
        if (!cpu) {
            platform->UnInit();
            platform = nullptr;
            sdkInitialized = false;
            FreeLibrary(platformModule);
            platformModule = nullptr;
            SetDllDirectoryW(nullptr);
            return -5;
        }
    } catch (...) {
        if (sdkInitialized && platform) {
            platform->UnInit();
        }
        cpu = nullptr;
        platform = nullptr;
        sdkInitialized = false;
        FreeLibrary(platformModule);
        platformModule = nullptr;
        SetDllDirectoryW(nullptr);
        return -6;
    }
    return 0;
}

int ak500_sdk_read(double* temperatureC) {
    if (!temperatureC || !cpu) {
        return -1;
    }

    try {
        CPUParameters parameters = {};
        const int result = cpu->GetCPUParameters(parameters);
        if (result != 0) {
            return -2;
        }
        const double value = parameters.dTemperature;
        if (!std::isfinite(value) || value <= 0.0 || value > 125.0) {
            return -3;
        }
        *temperatureC = value;
    } catch (...) {
        return -4;
    }
    return 0;
}

void ak500_sdk_close() {
    if (sdkInitialized && platform) {
        platform->UnInit();
    }
    cpu = nullptr;
    platform = nullptr;
    sdkInitialized = false;
    if (platformModule) {
        FreeLibrary(platformModule);
        platformModule = nullptr;
    }
    SetDllDirectoryW(nullptr);
}

} // extern "C"
