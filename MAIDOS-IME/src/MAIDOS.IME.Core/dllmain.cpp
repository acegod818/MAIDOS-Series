// dllmain.cpp : Defines the entry point for the DLL application.

#include "pch.h"

extern HMODULE g_hModule;

BOOL APIENTRY DllMain(HMODULE hModule,
    DWORD  ul_reason_for_call,
    LPVOID lpReserved
)
{
    UNREFERENCED_PARAMETER(lpReserved);

    switch (ul_reason_for_call)
    {
    case DLL_PROCESS_ATTACH:
        g_hModule = hModule;
        break;
    case DLL_THREAD_ATTACH:
    case DLL_THREAD_DETACH:
    case DLL_PROCESS_DETACH:
        if (ul_reason_for_call == DLL_PROCESS_DETACH) {
            g_hModule = nullptr;
        }
        break;
    }
    return TRUE;
}
