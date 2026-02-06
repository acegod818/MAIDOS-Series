#pragma once
#pragma warning(disable: 4819)
#include <iostream>
#include <string>
#include <fstream>
#include <chrono>
#include <iomanip>
#include <ctime>

// [MAIDOS-AUDIT] 符合憲法第 3 條：日誌審計系統
class MaidosLogger {
public:
    static void Log(const std::string& module, const std::string& message) {
        auto now = std::chrono::system_clock::now();
        auto now_t = std::chrono::system_clock::to_time_t(now);
        
        struct tm timeinfo;
        localtime_s(&timeinfo, &now_t);
        
        std::cout << "[MAIDOS-AUDIT][" << module << "] " << message << std::endl;
        
        std::ofstream logFile("maidos_driver.log", std::ios::app);
        if (logFile.is_open()) {
            logFile << "[" << std::put_time(&timeinfo, "%Y-%m-%d %H:%M:%S") << "]"
                    << "[AUDIT][" << module << "] " << message << std::endl;
        }
    }
};

#define AUDIT_LOG(module, msg) MaidosLogger::Log(module, msg)
#define AUDIT_ENTRY(func) AUDIT_LOG("NATIVE", "Entering " #func)
#define AUDIT_EXIT(func) AUDIT_LOG("NATIVE", "Exiting " #func)