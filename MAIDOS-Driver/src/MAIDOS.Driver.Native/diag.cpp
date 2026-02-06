#pragma warning(disable: 4819)
#include <windows.h>
#include <setupapi.h>
#include <cfgmgr32.h>
#include <string>
#include "logger.h"

#pragma comment(lib, "setupapi.lib")
#pragma comment(lib, "cfgmgr32.lib")

#ifndef CM_PROB_CODE43
#define CM_PROB_CODE43 (0x0000002B)
#endif

extern "C" {
    __declspec(dllexport) int get_device_problem_code(const char* deviceInstanceId) {
        AUDIT_ENTRY(get_device_problem_code);
        CONFIGRET cr;
        DEVINST devInst;
        ULONG status, problemCode;

        cr = CM_Locate_DevNodeA(&devInst, (DEVINSTID_A)deviceInstanceId, CM_LOCATE_DEVNODE_NORMAL);
        if (cr != CR_SUCCESS) {
            AUDIT_LOG("DIAG", "Device node not found: " + std::string(deviceInstanceId));
            return -1;
        }

        cr = CM_Get_DevNode_Status(&status, &problemCode, devInst, 0);
        if (cr != CR_SUCCESS) return 0;

        if (status & DN_HAS_PROBLEM) {
            AUDIT_LOG("DIAG", "Device " + std::string(deviceInstanceId) + " has problem: " + std::to_string(problemCode));
            return (int)problemCode;
        }

        AUDIT_EXIT(get_device_problem_code);
        return 0;
    }

    __declspec(dllexport) void get_problem_description_secure(int problemCode, char* buffer, int bufferSize) {
        std::string desc;
        switch (problemCode) {
            case CM_PROB_FAILED_INSTALL: desc = "驅動程式安裝失敗 (Code 28)"; break;
            case CM_PROB_OUT_OF_MEMORY: desc = "系統記憶體不足 (Code 3)"; break;
            case CM_PROB_CODE43: desc = "設備回報錯誤 (Code 43)"; break;
            case CM_PROB_DISABLED: desc = "設備已被禁用 (Code 22)"; break;
            case CM_PROB_NOT_CONFIGURED: desc = "設備未配置 (Code 1)"; break;
            case CM_PROB_FAILED_START: desc = "設備無法啟動 (Code 10)"; break;
            default: desc = "未知衝突或錯誤 (" + std::to_string(problemCode) + ")"; break;
        }
        strncpy_s(buffer, bufferSize, desc.c_str(), _TRUNCATE);
    }

    __declspec(dllexport) int get_device_irq(const char* deviceInstanceId) {
        AUDIT_ENTRY(get_device_irq);
        CONFIGRET cr;
        DEVINST devInst;
        LOG_CONF logConf;
        RES_DES resDes;
        BYTE resData[1024];

        cr = CM_Locate_DevNodeA(&devInst, (DEVINSTID_A)deviceInstanceId, CM_LOCATE_DEVNODE_NORMAL);
        if (cr != CR_SUCCESS) return -1;

        cr = CM_Get_First_Log_Conf(&logConf, devInst, ALLOC_LOG_CONF);
        if (cr != CR_SUCCESS) return 0;

        cr = CM_Get_Next_Res_Des(&resDes, logConf, ResType_IRQ, NULL, 0);
        if (cr == CR_SUCCESS) {
            cr = CM_Get_Res_Des_Data(resDes, resData, sizeof(resData), 0);
            if (cr == CR_SUCCESS) {
                unsigned int* pIrq = (unsigned int*)(resData + sizeof(unsigned int)); 
                int irq = (int)(*pIrq);
                AUDIT_LOG("DIAG", "Device IRQ: " + std::to_string(irq));
                CM_Free_Res_Des_Handle(resDes);
                CM_Free_Log_Conf_Handle(logConf);
                return irq;
            }
            CM_Free_Res_Des_Handle(resDes);
        }

        CM_Free_Log_Conf_Handle(logConf);
        AUDIT_EXIT(get_device_irq);
        return 0;
    }
}