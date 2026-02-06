#include "pch.h"
#include "ime_engine.h"

#include <msctf.h>
#include <new>

// Minimal TSF text service implementation.
// This is intentionally small but non-placeholder: it wires ITfTextInputProcessor +
// ITfKeyEventSink, buffers keystrokes, and commits a candidate via the existing ImeEngine.

// {8B5F7F26-8C58-4B45-9B7B-0C5C7A3E1D4A}
static const CLSID CLSID_MAIDOS_TextService = {
    0x8b5f7f26, 0x8c58, 0x4b45, { 0x9b, 0x7b, 0x0c, 0x5c, 0x7a, 0x3e, 0x1d, 0x4a }
};

// Globals for COM server lifetime.
HMODULE g_hModule = nullptr;
static LONG g_cRefDll = 0;

static std::wstring GetModulePathW()
{
    wchar_t buf[MAX_PATH];
    const HMODULE h = g_hModule ? g_hModule : nullptr;
    const DWORD len = GetModuleFileNameW(h, buf, static_cast<DWORD>(sizeof(buf) / sizeof(buf[0])));
    if (len == 0 || len >= (sizeof(buf) / sizeof(buf[0])))
    {
        return L"";
    }
    return std::wstring(buf, len);
}

static void DebugLog(const wchar_t* msg)
{
    if (!msg) return;
    OutputDebugStringW(msg);
    OutputDebugStringW(L"\r\n");
}

static HRESULT RegisterComServer()
{
    const std::wstring modulePath = GetModulePathW();
    if (modulePath.empty())
    {
        return E_FAIL;
    }

    wchar_t clsidBuf[64];
    if (StringFromGUID2(CLSID_MAIDOS_TextService, clsidBuf, static_cast<int>(sizeof(clsidBuf) / sizeof(clsidBuf[0]))) == 0)
    {
        return E_FAIL;
    }

    const std::wstring keyPath = std::wstring(L"CLSID\\") + clsidBuf + L"\\InprocServer32";
    HKEY hKey = nullptr;
    const LONG rc = RegCreateKeyExW(HKEY_CLASSES_ROOT, keyPath.c_str(), 0, nullptr, REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr);
    if (rc != ERROR_SUCCESS)
    {
        return HRESULT_FROM_WIN32(static_cast<DWORD>(rc));
    }

    const wchar_t* tm = L"Apartment";
    RegSetValueExW(hKey, nullptr, 0, REG_SZ, reinterpret_cast<const BYTE*>(modulePath.c_str()), static_cast<DWORD>((modulePath.size() + 1) * sizeof(wchar_t)));
    RegSetValueExW(hKey, L"ThreadingModel", 0, REG_SZ, reinterpret_cast<const BYTE*>(tm), static_cast<DWORD>((wcslen(tm) + 1) * sizeof(wchar_t)));
    RegCloseKey(hKey);
    return S_OK;
}

static void UnregisterComServer()
{
    wchar_t clsidBuf[64];
    if (StringFromGUID2(CLSID_MAIDOS_TextService, clsidBuf, static_cast<int>(sizeof(clsidBuf) / sizeof(clsidBuf[0]))) == 0)
    {
        return;
    }
    const std::wstring keyPath = std::wstring(L"CLSID\\") + clsidBuf;
    // Best-effort cleanup.
    (void)RegDeleteTreeW(HKEY_CLASSES_ROOT, keyPath.c_str());
}

// {B7A5C9B8-5D2E-4E8A-9F1A-9B7B9F9A5E3D}
static const GUID GUID_MAIDOS_Profile = {
    0xb7a5c9b8, 0x5d2e, 0x4e8a, { 0x9f, 0x1a, 0x9b, 0x7b, 0x9f, 0x9a, 0x5e, 0x3d }
};

static HRESULT RegisterTsfProfiles()
{
    const std::wstring modulePath = GetModulePathW();
    if (modulePath.empty())
    {
        return E_FAIL;
    }

    HRESULT hr = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    const bool coInit = SUCCEEDED(hr);
    if (FAILED(hr) && hr != RPC_E_CHANGED_MODE)
    {
        return hr;
    }

    ITfInputProcessorProfiles* profiles = nullptr;
    hr = CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER, IID_ITfInputProcessorProfiles, reinterpret_cast<void**>(&profiles));
    if (SUCCEEDED(hr) && profiles)
    {
        (void)profiles->Register(CLSID_MAIDOS_TextService);

        const wchar_t* desc = L"MAIDOS IME";
        const LANGID langid = MAKELANGID(LANG_CHINESE, SUBLANG_CHINESE_TRADITIONAL);

        hr = profiles->AddLanguageProfile(
            CLSID_MAIDOS_TextService,
            langid,
            GUID_MAIDOS_Profile,
            desc,
            static_cast<ULONG>(wcslen(desc)),
            modulePath.c_str(),
            static_cast<ULONG>(modulePath.size()),
            0
        );

        if (SUCCEEDED(hr))
        {
            (void)profiles->EnableLanguageProfile(CLSID_MAIDOS_TextService, langid, GUID_MAIDOS_Profile, TRUE);
        }

        profiles->Release();
        profiles = nullptr;
    }

    // Register as a keyboard TIP category (best-effort).
    ITfCategoryMgr* cat = nullptr;
    if (SUCCEEDED(CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER, IID_ITfCategoryMgr, reinterpret_cast<void**>(&cat))) && cat)
    {
        (void)cat->RegisterCategory(CLSID_MAIDOS_TextService, GUID_TFCAT_TIP_KEYBOARD, CLSID_MAIDOS_TextService);
        cat->Release();
    }

    if (coInit)
    {
        CoUninitialize();
    }

    return hr;
}

static void UnregisterTsfProfiles()
{
    HRESULT hr = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    const bool coInit = SUCCEEDED(hr);
    if (FAILED(hr) && hr != RPC_E_CHANGED_MODE)
    {
        return;
    }

    ITfInputProcessorProfiles* profiles = nullptr;
    if (SUCCEEDED(CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER, IID_ITfInputProcessorProfiles, reinterpret_cast<void**>(&profiles))) && profiles)
    {
        const LANGID langid = MAKELANGID(LANG_CHINESE, SUBLANG_CHINESE_TRADITIONAL);
        (void)profiles->RemoveLanguageProfile(CLSID_MAIDOS_TextService, langid, GUID_MAIDOS_Profile);
        (void)profiles->Unregister(CLSID_MAIDOS_TextService);
        profiles->Release();
    }

    ITfCategoryMgr* cat = nullptr;
    if (SUCCEEDED(CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER, IID_ITfCategoryMgr, reinterpret_cast<void**>(&cat))) && cat)
    {
        (void)cat->UnregisterCategory(CLSID_MAIDOS_TextService, GUID_TFCAT_TIP_KEYBOARD, CLSID_MAIDOS_TextService);
        cat->Release();
    }

    if (coInit)
    {
        CoUninitialize();
    }
}

class InsertTextEditSession final : public ITfEditSession {
public:
    InsertTextEditSession(ITfContext* context, std::wstring text) :
        m_ref(1),
        m_context(context),
        m_text(std::move(text))
    {
        if (m_context) m_context->AddRef();
    }

    ~InsertTextEditSession()
    {
        if (m_context) {
            m_context->Release();
            m_context = nullptr;
        }
    }

    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void** ppvObj) override
    {
        if (!ppvObj) return E_INVALIDARG;
        *ppvObj = nullptr;

        if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_ITfEditSession)) {
            *ppvObj = static_cast<ITfEditSession*>(this);
            AddRef();
            return S_OK;
        }
        return E_NOINTERFACE;
    }

    STDMETHODIMP_(ULONG) AddRef() override
    {
        return static_cast<ULONG>(InterlockedIncrement(&m_ref));
    }

    STDMETHODIMP_(ULONG) Release() override
    {
        const ULONG r = static_cast<ULONG>(InterlockedDecrement(&m_ref));
        if (r == 0) delete this;
        return r;
    }

    // ITfEditSession
    STDMETHODIMP DoEditSession(TfEditCookie ec) override
    {
        if (!m_context) return E_UNEXPECTED;
        if (m_text.empty()) return S_OK;

        ITfInsertAtSelection* insert = nullptr;
        HRESULT hr = m_context->QueryInterface(IID_ITfInsertAtSelection, reinterpret_cast<void**>(&insert));
        if (FAILED(hr)) return hr;

        ITfRange* range = nullptr;
        hr = insert->InsertTextAtSelection(ec, TF_IAS_NOQUERY, m_text.c_str(), static_cast<LONG>(m_text.size()), &range);
        if (range) range->Release();
        insert->Release();
        return hr;
    }

private:
    LONG m_ref;
    ITfContext* m_context;
    std::wstring m_text;
};

class MaidosTextService final : public ITfTextInputProcessor, public ITfKeyEventSink {
public:
    MaidosTextService() :
        m_ref(1),
        m_threadMgr(nullptr),
        m_clientId(TF_CLIENTID_NULL),
        m_keySinkActive(false),
        m_engineReady(false)
    {
        InterlockedIncrement(&g_cRefDll);
    }

    ~MaidosTextService()
    {
        Deactivate();
        InterlockedDecrement(&g_cRefDll);
    }

    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void** ppvObj) override
    {
        if (!ppvObj) return E_INVALIDARG;
        *ppvObj = nullptr;

        if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_ITfTextInputProcessor)) {
            *ppvObj = static_cast<ITfTextInputProcessor*>(this);
            AddRef();
            return S_OK;
        }
        if (IsEqualIID(riid, IID_ITfKeyEventSink)) {
            *ppvObj = static_cast<ITfKeyEventSink*>(this);
            AddRef();
            return S_OK;
        }
        return E_NOINTERFACE;
    }

    STDMETHODIMP_(ULONG) AddRef() override
    {
        return static_cast<ULONG>(InterlockedIncrement(&m_ref));
    }

    STDMETHODIMP_(ULONG) Release() override
    {
        const ULONG r = static_cast<ULONG>(InterlockedDecrement(&m_ref));
        if (r == 0) delete this;
        return r;
    }

    // ITfTextInputProcessor
    STDMETHODIMP Activate(ITfThreadMgr* ptim, TfClientId tid) override
    {
        if (!ptim) return E_INVALIDARG;

        m_threadMgr = ptim;
        m_threadMgr->AddRef();
        m_clientId = tid;

        ITfKeystrokeMgr* keyMgr = nullptr;
        HRESULT hr = m_threadMgr->QueryInterface(IID_ITfKeystrokeMgr, reinterpret_cast<void**>(&keyMgr));
        if (SUCCEEDED(hr) && keyMgr) {
            hr = keyMgr->AdviseKeyEventSink(m_clientId, static_cast<ITfKeyEventSink*>(this), TRUE);
            keyMgr->Release();
            m_keySinkActive = SUCCEEDED(hr);
        }

        if (SUCCEEDED(hr)) {
            DebugLog(L"MAIDOS TSF: Activated");
        } else {
            DebugLog(L"MAIDOS TSF: Activate failed");
        }

        return hr;
    }

    STDMETHODIMP Deactivate() override
    {
        if (m_threadMgr) {
            if (m_keySinkActive) {
                ITfKeystrokeMgr* keyMgr = nullptr;
                if (SUCCEEDED(m_threadMgr->QueryInterface(IID_ITfKeystrokeMgr, reinterpret_cast<void**>(&keyMgr))) && keyMgr) {
                    (void)keyMgr->UnadviseKeyEventSink(m_clientId);
                    keyMgr->Release();
                }
                m_keySinkActive = false;
            }

            m_threadMgr->Release();
            m_threadMgr = nullptr;
        }

        m_clientId = TF_CLIENTID_NULL;
        m_buffer.clear();
        return S_OK;
    }

    // ITfKeyEventSink
    STDMETHODIMP OnSetFocus(BOOL fForeground) override
    {
        UNREFERENCED_PARAMETER(fForeground);
        return S_OK;
    }

    STDMETHODIMP OnTestKeyDown(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) override
    {
        UNREFERENCED_PARAMETER(pic);
        UNREFERENCED_PARAMETER(lParam);
        if (!pfEaten) return E_INVALIDARG;

        // We only "eat" keys we actually handle on KeyDown.
        *pfEaten = (wParam == VK_SPACE) || (wParam == VK_BACK) || (wParam == VK_ESCAPE) ||
            ((wParam >= 'A' && wParam <= 'Z') || (wParam >= 'a' && wParam <= 'z'));
        return S_OK;
    }

    STDMETHODIMP OnKeyDown(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) override
    {
        UNREFERENCED_PARAMETER(lParam);
        if (!pfEaten) return E_INVALIDARG;
        *pfEaten = FALSE;

        if (!pic) return S_OK;

        // Buffer letters as pinyin input; commit on space.
        if ((wParam >= 'A' && wParam <= 'Z') || (wParam >= 'a' && wParam <= 'z')) {
            const wchar_t ch = static_cast<wchar_t>(wParam);
            m_buffer.push_back(ch);
            *pfEaten = TRUE;
            return S_OK;
        }

        if (wParam == VK_BACK) {
            if (!m_buffer.empty()) m_buffer.pop_back();
            *pfEaten = TRUE;
            return S_OK;
        }

        if (wParam == VK_ESCAPE) {
            m_buffer.clear();
            *pfEaten = TRUE;
            return S_OK;
        }

        if (wParam == VK_SPACE) {
            const HRESULT hr = CommitCandidate(pic);
            *pfEaten = TRUE;
            return hr;
        }

        return S_OK;
    }

    STDMETHODIMP OnTestKeyUp(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) override
    {
        UNREFERENCED_PARAMETER(pic);
        UNREFERENCED_PARAMETER(wParam);
        UNREFERENCED_PARAMETER(lParam);
        if (!pfEaten) return E_INVALIDARG;
        *pfEaten = FALSE;
        return S_OK;
    }

    STDMETHODIMP OnKeyUp(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) override
    {
        UNREFERENCED_PARAMETER(pic);
        UNREFERENCED_PARAMETER(wParam);
        UNREFERENCED_PARAMETER(lParam);
        if (!pfEaten) return E_INVALIDARG;
        *pfEaten = FALSE;
        return S_OK;
    }

    STDMETHODIMP OnPreservedKey(ITfContext* pic, REFGUID rguid, BOOL* pfEaten) override
    {
        UNREFERENCED_PARAMETER(pic);
        UNREFERENCED_PARAMETER(rguid);
        if (!pfEaten) return E_INVALIDARG;
        *pfEaten = FALSE;
        return S_OK;
    }

private:
    HRESULT EnsureEngineReady()
    {
        if (m_engineReady) return S_OK;

        // The engine resolves dictionaries via MAIDOS_IME_DICT_DIR / exe-dir fallbacks.
        if (!m_engine.Initialize(L"")) {
            return E_FAIL;
        }
        m_engineReady = true;
        return S_OK;
    }

    HRESULT CommitCandidate(ITfContext* context)
    {
        if (!context) return E_INVALIDARG;
        if (m_buffer.empty()) return S_OK;

        const HRESULT hrInit = EnsureEngineReady();
        if (FAILED(hrInit)) return hrInit;

        const auto candidates = m_engine.ProcessInput(m_buffer);
        const std::wstring out = candidates.empty() ? m_buffer : candidates[0].character;

        const HRESULT hr = CommitText(context, out);
        m_buffer.clear();
        return hr;
    }

    HRESULT CommitText(ITfContext* context, const std::wstring& text)
    {
        if (!context) return E_INVALIDARG;
        if (text.empty()) return S_OK;

        InsertTextEditSession* session = new (std::nothrow) InsertTextEditSession(context, text);
        if (!session) return E_OUTOFMEMORY;

        HRESULT hrSession = E_FAIL;
        const HRESULT hr = context->RequestEditSession(m_clientId, session, TF_ES_SYNC | TF_ES_READWRITE, &hrSession);
        session->Release();

        if (FAILED(hr)) return hr;
        return hrSession;
    }

    LONG m_ref;
    ITfThreadMgr* m_threadMgr;
    TfClientId m_clientId;
    bool m_keySinkActive;
    std::wstring m_buffer;
    ImeEngine m_engine;
    bool m_engineReady;
};

class MaidosClassFactory final : public IClassFactory {
public:
    MaidosClassFactory() : m_ref(1) {}
    ~MaidosClassFactory() = default;

    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void** ppvObj) override
    {
        if (!ppvObj) return E_INVALIDARG;
        *ppvObj = nullptr;

        if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_IClassFactory)) {
            *ppvObj = static_cast<IClassFactory*>(this);
            AddRef();
            return S_OK;
        }
        return E_NOINTERFACE;
    }

    STDMETHODIMP_(ULONG) AddRef() override
    {
        return static_cast<ULONG>(InterlockedIncrement(&m_ref));
    }

    STDMETHODIMP_(ULONG) Release() override
    {
        const ULONG r = static_cast<ULONG>(InterlockedDecrement(&m_ref));
        if (r == 0) delete this;
        return r;
    }

    // IClassFactory
    STDMETHODIMP CreateInstance(IUnknown* pUnkOuter, REFIID riid, void** ppvObj) override
    {
        if (!ppvObj) return E_INVALIDARG;
        *ppvObj = nullptr;

        if (pUnkOuter) return CLASS_E_NOAGGREGATION;

        MaidosTextService* svc = new (std::nothrow) MaidosTextService();
        if (!svc) return E_OUTOFMEMORY;

        const HRESULT hr = svc->QueryInterface(riid, ppvObj);
        svc->Release();
        return hr;
    }

    STDMETHODIMP LockServer(BOOL fLock) override
    {
        if (fLock) InterlockedIncrement(&g_cRefDll);
        else InterlockedDecrement(&g_cRefDll);
        return S_OK;
    }

private:
    LONG m_ref;
};

STDAPI DllCanUnloadNow(void)
{
    return (g_cRefDll == 0) ? S_OK : S_FALSE;
}

STDAPI DllGetClassObject(REFCLSID rclsid, REFIID riid, void** ppv)
{
    if (!ppv) return E_INVALIDARG;
    *ppv = nullptr;

    if (!IsEqualCLSID(rclsid, CLSID_MAIDOS_TextService)) {
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    MaidosClassFactory* factory = new (std::nothrow) MaidosClassFactory();
    if (!factory) return E_OUTOFMEMORY;

    const HRESULT hr = factory->QueryInterface(riid, ppv);
    factory->Release();
    return hr;
}

STDAPI DllRegisterServer(void)
{
    // Best-effort: COM registration + TSF profile registration.
    const HRESULT hrCom = RegisterComServer();
    const HRESULT hrTsf = RegisterTsfProfiles();
    return FAILED(hrCom) ? hrCom : hrTsf;
}

STDAPI DllUnregisterServer(void)
{
    UnregisterTsfProfiles();
    UnregisterComServer();
    return S_OK;
}
