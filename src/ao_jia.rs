use once_cell::sync::OnceCell;
use std::ptr;
use windows::{
    Win32::{
        Globalization::GetUserDefaultLCID,
        System::{
            Com::{
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoUninitialize, DISPATCH_METHOD, IDispatch, DISPPARAMS,
            },
            LibraryLoader::{GetProcAddress, LoadLibraryW},
            Variant::VARIANT,
        },
    },
    core::{GUID, HSTRING, PCSTR, PCWSTR},
};

// 对应 CARegJ 类
type FnSetDllPathW = unsafe extern "system" fn(PCWSTR, i32) -> i32;
static PFN_SET_DLL_PATH_W: OnceCell<Option<FnSetDllPathW>> = OnceCell::new();

pub fn set_dll_path(a_regj_path: String, ao_jia_path: String) -> i32 {
    let pfn = PFN_SET_DLL_PATH_W.get_or_init(|| unsafe {
        let a_regj_hstring = HSTRING::from(a_regj_path);
        let hmodule = LoadLibraryW(PCWSTR::from_raw(a_regj_hstring.as_ptr())).ok();
        hmodule.and_then(|h| {
            let proc_name = PCSTR::from_raw(r#"SetDllPathW "#.as_ptr());
            GetProcAddress(h, proc_name).map(|addr| std::mem::transmute(addr))
        })
    });

    if let Some(func) = pfn {
        unsafe {
            let ao_jia_hstring = HSTRING::from(ao_jia_path);
            func(PCWSTR::from_raw(ao_jia_hstring.as_ptr()), 0)
        }
    } else {
        0
    }
}

#[derive(Debug)]
pub struct AoJia {
    p_idispatch: Option<IDispatch>,
}

impl AoJia {
    const CLSID: GUID = GUID::from_values(
        0x4f27e588,
        0x5b1e,
        0x45b4,
        [0xad, 0x67, 0xe3, 0x2d, 0x45, 0xc4, 0xe9, 0xca],
    );

    pub fn new() -> windows::core::Result<Self> {
        unsafe {
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            if hr.is_err() {
                return Err(hr.into());
            }

            let idispatch: IDispatch = CoCreateInstance(&Self::CLSID, None, CLSCTX_INPROC_SERVER)?;

            Ok(Self {
                p_idispatch: Some(idispatch),
            })
        }
    }

    pub fn new_with_path(a_regj_path: String, ao_jia_path: String) -> windows::core::Result<Self> {
        set_dll_path(a_regj_path, ao_jia_path);
        Self::new()
    }

    fn call(
        &self,
        fun_name: &HSTRING,
        rgdispid: &mut i32,
        p_disp_params: &DISPPARAMS,
        p_var_result: &mut VARIANT,
    ) -> windows::core::Result<()> {
        unsafe {
            if *rgdispid == -1 {
                let names_ptr = PCWSTR::from_raw(fun_name.as_ptr());
                let names = [names_ptr];
                self.p_idispatch.as_ref().unwrap().GetIDsOfNames(
                    &GUID::default(),
                    names.as_ptr(),
                    1,
                    GetUserDefaultLCID(),
                    rgdispid,
                )?;
            }

            self.p_idispatch.as_ref().unwrap().Invoke(
                *rgdispid,
                &GUID::default(),
                GetUserDefaultLCID(),
                DISPATCH_METHOD,
                p_disp_params,
                Some(p_var_result),
                None,
                None,
            )
        }
    }
    #[allow(non_snake_case)]
    pub fn VerS(&self) -> windows::core::Result<String> {
        let fun_name = HSTRING::from("VerS");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();
        let disp_params = DISPPARAMS {
            rgvarg: ptr::null_mut(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 0,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;
        unsafe {
            let bstr = &var_result.Anonymous.Anonymous.Anonymous.bstrVal;
            Ok(bstr.to_string())
        }
    }
    #[allow(non_snake_case)]
    pub fn SetPath(&self, path: &str) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("SetPath");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();

        let mut args = [VARIANT::from(path)];

        let disp_params = DISPPARAMS {
            rgvarg: args.as_mut_ptr(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 1,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;
        Ok(unsafe { var_result.Anonymous.Anonymous.Anonymous.lVal })
    }
    #[allow(non_snake_case)]
    pub fn SetErrorMsg(&self, msg: i32) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("SetErrorMsg");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();

        let mut args = [VARIANT::from(msg)];

        let disp_params = DISPPARAMS {
            rgvarg: args.as_mut_ptr(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 1,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;
        Ok(unsafe { var_result.Anonymous.Anonymous.Anonymous.lVal })
    }
    #[allow(non_snake_case)]
    pub fn SetThread(&self, tn: i32) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("SetThread");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();

        let mut args = [VARIANT::from(tn)];

        let disp_params = DISPPARAMS {
            rgvarg: args.as_mut_ptr(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 1,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;
        Ok(unsafe { var_result.Anonymous.Anonymous.Anonymous.lVal })
    }
}

impl Drop for AoJia {
    fn drop(&mut self) {
        unsafe {
            // IDispatch implements Drop which will call Release internally
            // Just let it drop automatically
            self.p_idispatch.take();
            CoUninitialize();
        }
    }
}
