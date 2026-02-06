#include <windows.h>
#include <setupapi.h>
#include <newdev.h>
#include <srrestoreptapi.h>
#include "logger.h"

#pragma comment(lib, "setupapi.lib")
#pragma comment(lib, "newdev.lib")

#ifndef DIIRF_FORCE_INF
#define DIIRF_FORCE_INF (0x00000002)
#endif

typedef BOOL (WINAPI *PFN_SRSETRESTOREPOINTA)(PRESTOREPOINTINFOA, PSTATEMGRSTATUS);

extern "C" {
    __declspec(dllexport) int install_driver_native(const char* infPath) {
        AUDIT_ENTRY(install_driver_native);
        AUDIT_LOG("INSTALL", "INF Path: " + std::string(infPath));

        RESTOREPOINTINFOA rpInfo;
        STATEMGRSTATUS rpStatus;
        rpInfo.dwEventType = BEGIN_SYSTEM_CHANGE;
        rpInfo.dwRestorePtType = DEVICE_DRIVER_INSTALL;
        rpInfo.llSequenceNumber = 0;
        strcpy_s(rpInfo.szDescription, "MAIDOS Driver Installation Guard");

        HMODULE hSrClient = LoadLibraryA("srclient.dll");
        if (hSrClient) {
            PFN_SRSETRESTOREPOINTA pSrSetRestorePoint = (PFN_SRSETRESTOREPOINTA)GetProcAddress(hSrClient, "SRSetRestorePointA");
            if (pSrSetRestorePoint) {
                pSrSetRestorePoint(&rpInfo, &rpStatus);
                AUDIT_LOG("INSTALL", "System restore point created.");
            }
        }

        BOOL rebootRequired = FALSE;
        if (DiInstallDriverA(NULL, infPath, DIIRF_FORCE_INF, &rebootRequired)) {
            AUDIT_LOG("INSTALL", "DiInstallDriver success.");
            if (hSrClient) {
                PFN_SRSETRESTOREPOINTA pSrSetRestorePoint = (PFN_SRSETRESTOREPOINTA)GetProcAddress(hSrClient, "SRSetRestorePointA");
                if (pSrSetRestorePoint) {
                    rpInfo.dwEventType = END_SYSTEM_CHANGE;
                    rpInfo.llSequenceNumber = rpStatus.llSequenceNumber;
                    pSrSetRestorePoint(&rpInfo, &rpStatus);
                }
                FreeLibrary(hSrClient);
            }
            AUDIT_EXIT(install_driver_native);
            return rebootRequired ? 2 : 1;
        } else {
            DWORD err = GetLastError();
            AUDIT_LOG("INSTALL", "DiInstallDriver failed with error: " + std::to_string(err));
            if (hSrClient) FreeLibrary(hSrClient);
            AUDIT_EXIT(install_driver_native);
            return -(int)err;
        }
    }
}