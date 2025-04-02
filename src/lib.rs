use once_cell::sync::OnceCell;
use std::ptr;
use windows::{
    Win32::{
        Globalization::GetUserDefaultLCID,
        System::{
            Com::{
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoUninitialize, DISPATCH_METHOD, DISPPARAMS, IDispatch,
            },
            LibraryLoader::{GetProcAddress, LoadLibraryW},
            Variant::{
                VAR_CHANGE_FLAGS, VARENUM, VARIANT, VARIANT_0_0, VT_BOOL, VT_BSTR, VT_BYREF, VT_I4,
                VT_VARIANT, VariantChangeType, VariantClear,
            },
        },
    },
    core::{GUID, HSTRING, PCSTR, PCWSTR},
};

use std::mem::ManuallyDrop;

pub trait VariantExt {
    fn by_ref(var_val: *mut VARIANT) -> VARIANT;
    fn to_i32(&self) -> windows::core::Result<i32>;
    fn to_string(&self) -> windows::core::Result<String>;
    fn to_bool(&self) -> windows::core::Result<bool>;
}

impl VariantExt for VARIANT {
    fn by_ref(var_val: *mut VARIANT) -> VARIANT {
        let mut variant = VARIANT::default();
        let mut v00 = VARIANT_0_0 {
            vt: VARENUM(VT_BYREF.0 | VT_VARIANT.0),
            ..Default::default()
        };
        v00.Anonymous.pvarVal = var_val;
        variant.Anonymous.Anonymous = ManuallyDrop::new(v00);
        variant
    }
    fn to_i32(&self) -> windows::core::Result<i32> {
        unsafe {
            let mut new = VARIANT::default();
            VariantChangeType(&mut new, self, VAR_CHANGE_FLAGS(0), VT_I4)?;
            let v00 = &new.Anonymous.Anonymous;
            let n = v00.Anonymous.lVal;
            VariantClear(&mut new)?;
            Ok(n)
        }
    }
    fn to_string(&self) -> windows::core::Result<String> {
        unsafe {
            let mut new = VARIANT::default();
            VariantChangeType(&mut new, self, VAR_CHANGE_FLAGS(0), VT_BSTR)?;
            let v00 = &new.Anonymous.Anonymous;
            let str = v00.Anonymous.bstrVal.to_string();
            VariantClear(&mut new)?;
            Ok(str)
        }
    }
    fn to_bool(&self) -> windows::core::Result<bool> {
        unsafe {
            let mut new = VARIANT::default();
            VariantChangeType(&mut new, self, VAR_CHANGE_FLAGS(0), VT_BOOL)?;
            let v00 = &new.Anonymous.Anonymous;
            let b = v00.Anonymous.boolVal.as_bool();
            VariantClear(&mut new)?;
            Ok(b)
        }
    }
}

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
    pub fn SetPath(&self, Path: &str) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("SetPath");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();
        let mut args = [VARIANT::from(Path)];
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
    pub fn SetErrorMsg(&self, Msg: i32) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("SetErrorMsg");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();
        let mut args = [VARIANT::from(Msg)];
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
    pub fn SetThread(&self, TN: i32) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("SetThread");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();
        let mut args = [VARIANT::from(TN)];
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
    pub fn GetModulePath(
        &self,
        PID: i32,
        Hwnd: i32,
        MN: &str,
        Type: i32,
    ) -> windows::core::Result<String> {
        let fun_name = HSTRING::from("GetModulePath");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();
        let mut args = [
            VARIANT::from(Type),
            VARIANT::from(MN),
            VARIANT::from(Hwnd),
            VARIANT::from(PID),
        ];
        let disp_params = DISPPARAMS {
            rgvarg: args.as_mut_ptr(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 4,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;
        Ok(unsafe { var_result.Anonymous.Anonymous.Anonymous.bstrVal.to_string() })
    }
    #[allow(non_snake_case)]
    pub fn GetMachineCode(&self) -> windows::core::Result<String> {
        let fun_name = HSTRING::from("GetMachineCode");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();
        let disp_params = DISPPARAMS {
            rgvarg: ptr::null_mut(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 0,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;
        Ok(unsafe { var_result.Anonymous.Anonymous.Anonymous.bstrVal.to_string() })
    }
    #[allow(non_snake_case)]
    pub fn GetOs(
        &self,
        SV: &mut String,
        SVN: &mut String,
        LVBN: &mut i32,
        SDir: &mut String,
        Type: i32,
    ) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("GetOs");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();

        // Create VARIANT instances for the out parameters
        let mut v = VARIANT::default();
        let mut vn = VARIANT::default();
        let mut vbn = VARIANT::default();
        let mut dir = VARIANT::default();

        // Set up the arguments array - note the reverse order compared to C++
        let mut args = [
            VARIANT::from(Type),
            VARIANT::by_ref(&mut dir as *mut VARIANT), // dir
            VARIANT::by_ref(&mut vbn as *mut VARIANT), // vbn
            VARIANT::by_ref(&mut vn as *mut VARIANT),  // vn
            VARIANT::by_ref(&mut v as *mut VARIANT),   // v
        ];

        let disp_params = DISPPARAMS {
            rgvarg: args.as_mut_ptr(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 5,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;

        // Extract the values and assign to the out parameters
        unsafe {
            *SV = v.Anonymous.Anonymous.Anonymous.bstrVal.to_string();
            *SVN = vn.Anonymous.Anonymous.Anonymous.bstrVal.to_string();
            *LVBN = vbn.Anonymous.Anonymous.Anonymous.lVal;
            *SDir = dir.Anonymous.Anonymous.Anonymous.bstrVal.to_string();
            Ok(var_result.Anonymous.Anonymous.Anonymous.lVal)
        }
    }
    #[allow(non_snake_case, clippy::too_many_arguments)]
    pub fn FindWindow(
        &self,
        Parent: i32,
        ProName: &str,
        ProId: i32,
        Class: &str,
        Title: &str,
        Type: i32,
        T: i32,
    ) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("FindWindow");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();
        let mut args = [
            VARIANT::from(T),
            VARIANT::from(Type),
            VARIANT::from(Title),
            VARIANT::from(Class),
            VARIANT::from(ProId),
            VARIANT::from(ProName),
            VARIANT::from(Parent),
        ];
        let disp_params = DISPPARAMS {
            rgvarg: args.as_mut_ptr(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 7,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;
        Ok(unsafe { var_result.Anonymous.Anonymous.Anonymous.lVal })
    }
    #[allow(non_snake_case, clippy::too_many_arguments)]
    pub fn CreateWindows(
        &self,
        x: i32,
        y: i32,
        Width: i32,
        Height: i32,
        EWidth: i32,
        EHeight: i32,
        Type: i32,
    ) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("CreateWindows");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();
        let mut args = [
            VARIANT::from(Type),
            VARIANT::from(EHeight),
            VARIANT::from(EWidth),
            VARIANT::from(Height),
            VARIANT::from(Width),
            VARIANT::from(y),
            VARIANT::from(x),
        ];
        let disp_params = DISPPARAMS {
            rgvarg: args.as_mut_ptr(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 7,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;
        Ok(unsafe { var_result.Anonymous.Anonymous.Anonymous.lVal })
    }
    #[allow(non_snake_case)]
    pub fn GetRemoteProcAddress(
        &self,
        PID: i32,
        Hwnd: i32,
        MN: &str,
        Func: &str,
    ) -> windows::core::Result<i64> {
        let fun_name = HSTRING::from("GetRemoteProcAddress");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();
        let mut args = [
            VARIANT::from(Func),
            VARIANT::from(MN),
            VARIANT::from(Hwnd),
            VARIANT::from(PID),
        ];
        let disp_params = DISPPARAMS {
            rgvarg: args.as_mut_ptr(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 4,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;
        Ok(unsafe { var_result.Anonymous.Anonymous.Anonymous.llVal })
    }
    #[allow(non_snake_case)]
    pub fn KQHouTai(
        &self,
        Hwnd: i32,
        Screen: &str,
        Keyboard: &str,
        Mouse: &str,
        Flag: &str,
        Type: i32,
    ) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("KQHouTai");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();
        let mut args = [
            VARIANT::from(Type),
            VARIANT::from(Flag),
            VARIANT::from(Mouse),
            VARIANT::from(Keyboard),
            VARIANT::from(Screen),
            VARIANT::from(Hwnd),
        ];
        let disp_params = DISPPARAMS {
            rgvarg: args.as_mut_ptr(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 6,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;
        var_result.to_i32()
    }
    #[allow(non_snake_case)]
    pub fn GBHouTai(&self) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("GBHouTai");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();
        let disp_params = DISPPARAMS {
            rgvarg: ptr::null_mut(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 0,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;
        var_result.to_i32()
    }
    #[allow(non_snake_case)]
    pub fn GetCPU(&self, Type: &mut String, CPUID: &mut String) -> windows::core::Result<i32> {
        let fun_name = HSTRING::from("GetCPU");
        let mut disp_id = -1;
        let mut var_result = VARIANT::default();

        // Create VARIANT instances for the out parameters
        let mut ty = VARIANT::default();
        let mut id = VARIANT::default();

        // Set up the arguments array - note the reverse order compared to C++
        let mut args = [
            VARIANT::by_ref(&mut id as *mut VARIANT),
            VARIANT::by_ref(&mut ty as *mut VARIANT),
        ];

        let disp_params = DISPPARAMS {
            rgvarg: args.as_mut_ptr(),
            rgdispidNamedArgs: ptr::null_mut(),
            cArgs: 2,
            cNamedArgs: 0,
        };

        self.call(&fun_name, &mut disp_id, &disp_params, &mut var_result)?;

        // Extract the values and assign to the out parameters
        *Type = VariantExt::to_string(&ty).unwrap();
        *CPUID = VariantExt::to_string(&id).unwrap();
        var_result.to_i32()
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
