/// Example:
///
/// Note: the protect method could be identified as unused code on the Android
/// side and stripped by ProGuard, you may need to add a keep rule to let
/// ProGuard knows we need the code, e.g.:
///
/// -keep class com.leaf.and.aleaf.** { *; }
///
/// // Sets a callback method to protect sockets.
/// //
/// // Expects a method with the given name and signature `(I)Z`.
/// #[allow(non_snake_case)]
/// #[no_mangle]
/// pub unsafe extern "system" fn Java_com_leaf_and_aleaf_SimpleVpnService_setProtectSocketCallback(
///     mut env: JNIEnv,
///     class: JClass,
///     name: JString,
/// ) {
///     let Ok(name) = env.get_string(&name) else {
///         return;
///     };
///     let name: String = name.into();
///     if let Ok(class_g) = env.new_global_ref(class) {
///         leaf::mobile::callback::android::set_protect_socket_callback(class_g, name);
///     }
/// }
///
/// #[allow(non_snake_case)]
/// #[no_mangle]
/// pub unsafe extern "system" fn JNI_OnLoad(vm: JavaVM, _: *mut std::os::raw::c_void) -> jint {
///     leaf::mobile::callback::android::set_jvm(vm);
///     JNI_VERSION_1_6
/// }
///
/// #[allow(non_snake_case)]
/// #[no_mangle]
/// pub unsafe extern "system" fn JNI_OnUnload(vm: JavaVM, _: *mut std::os::raw::c_void) {
///     leaf::mobile::callback::android::unset_protect_socket_callback();
///     leaf::mobile::callback::android::unset_jvm();
/// }
#[cfg(target_os = "android")]
pub mod android {
    use std::os::unix::io::RawFd;

    use anyhow::{anyhow, Result};
    use jni::{objects::*, JavaVM};
    use std::sync::RwLock;
    use jni::strings::JNIString;

    static CALLBACK_PROTECT_SOCKET: RwLock<Option<CallbackProtectSocket>> = RwLock::new(None);

    struct CallbackProtectSocket {
        obj: Global<JObject<'static>>,
        name: String,
    }

    pub fn set_protect_socket_callback(obj: Global<JObject>, name: String) {
        *CALLBACK_PROTECT_SOCKET.write().unwrap() = Some(CallbackProtectSocket { obj, name });
    }

    pub fn unset_protect_socket_callback() {
        *CALLBACK_PROTECT_SOCKET.write().unwrap() = None;
    }

    pub fn is_protect_socket_callback_set() -> bool {
        (*CALLBACK_PROTECT_SOCKET.read().unwrap()).is_some()
    }

    pub fn protect_socket(fd: RawFd) -> Result<()> {
        // let jvm_g = JVM.read().unwrap();
        // let Some(vm) = jvm_g.as_ref() else {
        //     return Err(anyhow!("Java VM not set"));
        // };
        let vm = JavaVM::singleton()?;
        let cb_g = CALLBACK_PROTECT_SOCKET.read().unwrap();
        let Some(cb) = (*cb_g).as_ref() else {
            return Err(anyhow!("protect socket callback not set"));
        };
        vm.attach_current_thread(|mut env| -> Result<()> {
            let success = env.call_method(
                &cb.obj,
                JNIString::new(&cb.name),
                JNIString::new("(I)Z"),
                &[JValue::Int(fd as i32)],
            ).unwrap().z().unwrap();
            if !success {
                return Err(anyhow!("protect socket failed"));
            }
            Ok(())
        })?;
        Ok(())
    }
}
