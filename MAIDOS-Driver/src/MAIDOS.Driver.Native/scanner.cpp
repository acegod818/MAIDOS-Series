#pragma warning(disable: 4819)
#include <windows.h>
#include <setupapi.h>
#include <devguid.h>
#include <cfgmgr32.h>
#include <vector>
#include <string>
#include <cstdio>
#include "logger.h"

#pragma comment(lib, "setupapi.lib")
#pragma comment(lib, "cfgmgr32.lib")

// [MAIDOS-AUDIT] Entry: Hardware Enumeration (Universal Secure)
// 符合憲法第 3 條：全流程日誌審計

struct NativeDeviceInfo {
    char id[512];
    char name[512];
    char vendor[512];
    char version[64];
    char status[64];
};

void GetDeviceProperty(HDEVINFO hDevInfo, PSP_DEVINFO_DATA pDevInfoData, DWORD Property, char* buffer, DWORD bufferSize) {
    DWORD dataType;
    DWORD requiredSize = 0;
    SetupDiGetDeviceRegistryPropertyA(hDevInfo, pDevInfoData, Property, &dataType, NULL, 0, &requiredSize);
    if (requiredSize > 0) {
        std::vector<char> tempBuffer(requiredSize);
        if (SetupDiGetDeviceRegistryPropertyA(hDevInfo, pDevInfoData, Property, &dataType, (PBYTE)tempBuffer.data(), requiredSize, NULL)) {
            strncpy_s(buffer, bufferSize, tempBuffer.data(), _TRUNCATE);
            return;
        }
    }
    strncpy_s(buffer, bufferSize, "Unknown", _TRUNCATE);
}

extern "C" {
    __declspec(dllexport) int scan_hardware_native(NativeDeviceInfo* buffer, int maxCount) {
        AUDIT_ENTRY(scan_hardware_native);
        SP_DEVINFO_DATA devInfoData;
        devInfoData.cbSize = sizeof(SP_DEVINFO_DATA);

        HDEVINFO hDevInfo = SetupDiClassDevs(NULL, NULL, NULL, DIGCF_ALLCLASSES | DIGCF_PRESENT);
        if (hDevInfo == INVALID_HANDLE_VALUE) {
            AUDIT_LOG("SCAN", "Failed to get device list.");
            return -1;
        }

        int count = 0;
        for (DWORD i = 0; SetupDiEnumDeviceInfo(hDevInfo, i, &devInfoData) && count < maxCount; i++) {
            GetDeviceProperty(hDevInfo, &devInfoData, SPDRP_FRIENDLYNAME, buffer[count].name, 512);
            if (strcmp(buffer[count].name, "Unknown") == 0) {
                GetDeviceProperty(hDevInfo, &devInfoData, SPDRP_DEVICEDESC, buffer[count].name, 512);
            }
            GetDeviceProperty(hDevInfo, &devInfoData, SPDRP_HARDWAREID, buffer[count].id, 512);
            GetDeviceProperty(hDevInfo, &devInfoData, SPDRP_MFG, buffer[count].vendor, 512);
            
            // [MAIDOS-AUDIT] 獲取真實版本與狀態邏輯
            GetDeviceProperty(hDevInfo, &devInfoData, SPDRP_DRIVER, buffer[count].version, 64);
            
            DWORD status, problem;
            if (CM_Get_DevNode_Status(&status, &problem, devInfoData.DevInst, 0) == CR_SUCCESS) {
                if (status & DN_HAS_PROBLEM) {
                    sprintf_s(buffer[count].status, 64, "Error(Code %d)", problem);
                } else {
                    strncpy_s(buffer[count].status, 64, "Running", _TRUNCATE);
                }
            } else {
                strncpy_s(buffer[count].status, 64, "Unknown", _TRUNCATE);
            }
            
            count++;
        }

        AUDIT_LOG("SCAN", "Successfully scanned " + std::to_string(count) + " devices.");
        SetupDiDestroyDeviceInfoList(hDevInfo);
        AUDIT_EXIT(scan_hardware_native);
        return count;
    }
}