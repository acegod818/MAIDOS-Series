#pragma warning(disable: 4819)
#include <windows.h>
#include <setupapi.h>
#include <string>
#include "logger.h"

#pragma comment(lib, "setupapi.lib")

extern "C" {
    __declspec(dllexport) int backup_driver_native(const char* destinationPath) {
        AUDIT_ENTRY(backup_driver_native);
        AUDIT_LOG("BACKUP", "Target path: " + std::string(destinationPath));

        if (!CreateDirectoryA(destinationPath, NULL) && GetLastError() != ERROR_ALREADY_EXISTS) {
            AUDIT_LOG("BACKUP", "Failed to create directory.");
            return -1;
        }

        std::string cmd = "pnputil.exe /export-driver * \"" + std::string(destinationPath) + "\"";
        
        STARTUPINFOA si = { sizeof(si) };
        PROCESS_INFORMATION pi;
        
        if (CreateProcessA(NULL, (LPSTR)cmd.c_str(), NULL, NULL, FALSE, CREATE_NO_WINDOW, NULL, NULL, &si, &pi)) {
            WaitForSingleObject(pi.hProcess, INFINITE);
            DWORD exitCode;
            GetExitCodeProcess(pi.hProcess, &exitCode);
            CloseHandle(pi.hProcess);
            CloseHandle(pi.hThread);
            
            if (exitCode == 0) {
                AUDIT_LOG("BACKUP", "Driver export successful.");
                AUDIT_EXIT(backup_driver_native);
                return 1;
            } else {
                AUDIT_LOG("BACKUP", "PnPUtil exited with error: " + std::to_string(exitCode));
                return -(int)exitCode;
            }
        } else {
            DWORD err = GetLastError();
            AUDIT_LOG("BACKUP", "CreateProcess failed: " + std::to_string(err));
            AUDIT_EXIT(backup_driver_native);
            return -(int)err;
        }
    }
}