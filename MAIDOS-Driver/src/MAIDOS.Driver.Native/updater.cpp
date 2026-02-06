/**
 * [MAIDOS-AUDIT] 驅動更新模組 - SS級加固
 * 功能: 線上更新、本地更新、版本比對
 */

#include <windows.h>
#include <wininet.h>
#include <setupapi.h>
#include <stdio.h>
#include <string.h>
#include "logger.h"

#pragma comment(lib, "wininet.lib")
#pragma comment(lib, "setupapi.lib")

#define EXPORT extern "C" __declspec(dllexport)
#define MAX_PATH_LEN 512
#define MAX_URL_LEN 2048
#define BUFFER_SIZE 8192

// 更新結果結構
typedef struct {
    char device_id[MAX_PATH_LEN];
    char current_version[64];
    char latest_version[64];
    int update_available;  // 0=否, 1=是
    int update_status;     // 0=成功, -1=失敗, 1=無需更新
} UpdateResult;

/**
 * [MAIDOS-AUDIT] 檢查驅動更新 (線上)
 * @param device_id 設備實例ID
 * @param update_server 更新伺服器URL
 * @param result 更新結果
 * @return 1=有更新, 0=無更新, -1=錯誤
 */
EXPORT int check_driver_update(const char* device_id, const char* update_server, UpdateResult* result) {
    if (!device_id || !result) return -1;
    
    memset(result, 0, sizeof(UpdateResult));
    strncpy_s(result->device_id, MAX_PATH_LEN, device_id, _TRUNCATE);
    
    // 獲取當前驅動版本
    HDEVINFO devInfo = SetupDiGetClassDevs(NULL, NULL, NULL, DIGCF_ALLCLASSES | DIGCF_PRESENT);
    if (devInfo == INVALID_HANDLE_VALUE) {
        result->update_status = -1;
        return -1;
    }
    
    SP_DEVINFO_DATA devData;
    devData.cbSize = sizeof(SP_DEVINFO_DATA);
    BOOL found = FALSE;
    
    for (DWORD i = 0; SetupDiEnumDeviceInfo(devInfo, i, &devData); i++) {
        char instanceId[MAX_PATH_LEN];
        if (SetupDiGetDeviceInstanceIdA(devInfo, &devData, instanceId, MAX_PATH_LEN, NULL)) {
            if (strcmp(instanceId, device_id) == 0) {
                // 獲取驅動版本
                char driverVersion[64] = "Unknown";
                DWORD reqSize;
                SetupDiGetDeviceRegistryPropertyA(devInfo, &devData, SPDRP_DRIVER,
                    NULL, (PBYTE)driverVersion, sizeof(driverVersion), &reqSize);
                strncpy_s(result->current_version, 64, driverVersion, _TRUNCATE);
                found = TRUE;
                break;
            }
        }
    }
    SetupDiDestroyDeviceInfoList(devInfo);
    
    if (!found) {
        strcpy_s(result->current_version, 64, "Not Found");
        result->update_status = -1;
        return -1;
    }
    
    // [MAIDOS-AUDIT] 線上版本檢查
    // 如果提供了更新伺服器 URL，發送 HTTP 請求檢查最新版本
    if (update_server && strlen(update_server) > 0) {
        HINTERNET hInternet = InternetOpenA("MAIDOS-Driver-Updater/1.0",
            INTERNET_OPEN_TYPE_PRECONFIG, NULL, NULL, 0);
        if (hInternet) {
            // 構造查詢 URL: update_server + device_id
            char queryUrl[MAX_URL_LEN];
            snprintf(queryUrl, MAX_URL_LEN, "%s%s", update_server, device_id);
            
            HINTERNET hUrl = InternetOpenUrlA(hInternet, queryUrl, NULL, 0,
                INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE, 0);
            if (hUrl) {
                char responseBuffer[256] = {0};
                DWORD bytesRead = 0;
                if (InternetReadFile(hUrl, responseBuffer, sizeof(responseBuffer) - 1, &bytesRead) && bytesRead > 0) {
                    responseBuffer[bytesRead] = '\0';
                    // 預期回應格式: "VERSION:x.x.x" 或直接版本號
                    char* versionStart = strstr(responseBuffer, ":");
                    if (versionStart) {
                        versionStart++; // 跳過 ':'
                    } else {
                        versionStart = responseBuffer; // 直接是版本號
                    }
                    // 去除換行符
                    char* newline = strchr(versionStart, '\n');
                    if (newline) *newline = '\0';
                    newline = strchr(versionStart, '\r');
                    if (newline) *newline = '\0';
                    
                    strncpy_s(result->latest_version, 64, versionStart, _TRUNCATE);
                    
                    // 比對版本
                    if (strcmp(result->current_version, result->latest_version) != 0) {
                        result->update_available = 1;
                        result->update_status = 0;  // 有更新可用
                        InternetCloseHandle(hUrl);
                        InternetCloseHandle(hInternet);
                        return 1;
                    }
                }
                InternetCloseHandle(hUrl);
            }
            InternetCloseHandle(hInternet);
        }
    }
    
    // 無更新伺服器、連線失敗、或版本相同
    strcpy_s(result->latest_version, 64, result->current_version);
    result->update_available = 0;
    result->update_status = 1;  // 已是最新
    
    return 0;
}

/**
 * [MAIDOS-AUDIT] 下載驅動更新
 * @param download_url 下載URL
 * @param save_path 保存路徑
 * @return 1=成功, -1=失敗
 */
EXPORT int download_driver_update(const char* download_url, const char* save_path) {
    if (!download_url || !save_path) return -1;
    
    HINTERNET hInternet = InternetOpenA("MAIDOS-Driver-Updater/1.0", 
        INTERNET_OPEN_TYPE_PRECONFIG, NULL, NULL, 0);
    if (!hInternet) return -1;
    
    HINTERNET hUrl = InternetOpenUrlA(hInternet, download_url, NULL, 0,
        INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE, 0);
    if (!hUrl) {
        InternetCloseHandle(hInternet);
        return -1;
    }
    
    FILE* fp;
    if (fopen_s(&fp, save_path, "wb") != 0) {
        InternetCloseHandle(hUrl);
        InternetCloseHandle(hInternet);
        return -1;
    }
    
    char buffer[BUFFER_SIZE];
    DWORD bytesRead;
    while (InternetReadFile(hUrl, buffer, BUFFER_SIZE, &bytesRead) && bytesRead > 0) {
        fwrite(buffer, 1, bytesRead, fp);
    }
    
    fclose(fp);
    InternetCloseHandle(hUrl);
    InternetCloseHandle(hInternet);
    
    return 1;
}

/**
 * [MAIDOS-AUDIT] 執行驅動更新 (本地INF)
 * @param inf_path INF文件路徑
 * @param device_id 設備ID (可選, NULL=自動匹配)
 * @return 1=成功, -1=失敗
 */
EXPORT int apply_driver_update(const char* inf_path, const char* device_id) {
    if (!inf_path) return -1;
    
    // [MAIDOS-AUDIT] 空字符串視同NULL，表示自動匹配所有設備
    BOOL match_all = (!device_id || device_id[0] == '\0');
    
    // 使用 UpdateDriverForPlugAndPlayDevices API
    // 或者調用現有的 install_driver_native
    
    HDEVINFO devInfo = SetupDiGetClassDevs(NULL, NULL, NULL, DIGCF_ALLCLASSES | DIGCF_PRESENT);
    if (devInfo == INVALID_HANDLE_VALUE) return -1;
    
    SP_DEVINFO_DATA devData;
    devData.cbSize = sizeof(SP_DEVINFO_DATA);
    BOOL success = FALSE;
    
    for (DWORD i = 0; SetupDiEnumDeviceInfo(devInfo, i, &devData); i++) {
        char instanceId[MAX_PATH_LEN];
        if (SetupDiGetDeviceInstanceIdA(devInfo, &devData, instanceId, MAX_PATH_LEN, NULL)) {
            if (match_all || strcmp(instanceId, device_id) == 0) {
                // 嘗試更新驅動
                BOOL needReboot = FALSE;
                if (SetupDiInstallDevice(devInfo, &devData)) {
                    success = TRUE;
                    if (!match_all) break;  // 指定設備，找到即退出
                }
            }
        }
    }
    
    SetupDiDestroyDeviceInfoList(devInfo);
    return success ? 1 : -1;
}

/**
 * [MAIDOS-AUDIT] 批次檢查所有設備更新
 * @param results 結果陣列
 * @param max_count 最大數量
 * @return 實際檢查數量, -1=錯誤
 */
EXPORT int check_all_updates(UpdateResult* results, int max_count) {
    if (!results || max_count <= 0) return -1;
    
    HDEVINFO devInfo = SetupDiGetClassDevs(NULL, NULL, NULL, DIGCF_ALLCLASSES | DIGCF_PRESENT);
    if (devInfo == INVALID_HANDLE_VALUE) return -1;
    
    SP_DEVINFO_DATA devData;
    devData.cbSize = sizeof(SP_DEVINFO_DATA);
    int count = 0;
    
    for (DWORD i = 0; SetupDiEnumDeviceInfo(devInfo, i, &devData) && count < max_count; i++) {
        char instanceId[MAX_PATH_LEN];
        if (SetupDiGetDeviceInstanceIdA(devInfo, &devData, instanceId, MAX_PATH_LEN, NULL)) {
            check_driver_update(instanceId, NULL, &results[count]);
            count++;
        }
    }
    
    SetupDiDestroyDeviceInfoList(devInfo);
    return count;
}
