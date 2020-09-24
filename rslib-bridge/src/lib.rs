use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::{jbyteArray, jint, jlong};

use anki::backend::{init_backend, anki_error_to_proto_error, Backend as RustBackend};

use anki::backend_proto as pb;
use pb::Empty;
use pb::OpenCollectionIn;
use crate::sqlite::open_collection_ankidroid;

// allows encode/decode
use prost::Message;

mod mmap64;
mod sqlite;

struct Backend {
    backend: RustBackend,
}
impl Backend {
    pub fn new(backend: RustBackend) -> Backend {
        Backend { backend }
    }
}

#[no_mangle]
pub unsafe extern "C" fn Java_net_ankiweb_rsdroid_NativeMethods_openBackend(
    env: JNIEnv,
    _: JClass,
    args: jbyteArray) -> jlong {

    let rust_backend = init_backend(env.convert_byte_array(args).unwrap().as_slice()).unwrap();
    let backend = Backend::new(rust_backend);


    Box::into_raw(Box::new(backend)) as jlong
}




#[no_mangle]
pub unsafe extern "C" fn Java_net_ankiweb_rsdroid_NativeMethods_openCollection(
    env: JNIEnv,
    _: JClass,
    backend : jlong,
    args: jbyteArray) -> jbyteArray {

    let backend = &mut *(backend as *mut Backend);
    let in_bytes = env.convert_byte_array(args).unwrap();

    let command = OpenCollectionIn::decode(in_bytes.as_slice()).unwrap();

    let ret = open_collection_ankidroid(&backend.backend,command).and_then(|empty| {
        let mut out_bytes = Vec::new();
        empty.encode(&mut out_bytes)?;
        Ok(out_bytes)
    }).map_err(|err| {
        let backend_err = anki_error_to_proto_error(err, &backend.backend.i18n);
        let mut bytes = Vec::new();
        backend_err.encode(&mut bytes).unwrap();
        bytes
    });

    match ret {
        Ok(_s) => env.byte_array_from_slice(_s.as_slice()).unwrap(),
        Err(_err) => env.byte_array_from_slice(_err.as_slice()).unwrap(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn Java_net_ankiweb_rsdroid_NativeMethods_command(
    env: JNIEnv,
    _: JClass,
    backend : jlong,
    command: jint,
    args: jbyteArray,
) -> jbyteArray {

    let backend = &mut *(backend as *mut Backend);
    let command: u32 = command as u32;
    let in_bytes = env.convert_byte_array(args).unwrap();

    // We might want to later change this to append a bit to the head of the stream to specify
    // the return type.
    match backend.backend.run_command_bytes(command, in_bytes.as_slice()) {
        Ok(_s) => env.byte_array_from_slice(_s.as_slice()).unwrap(),
        Err(_err) => env.byte_array_from_slice(_err.as_slice()).unwrap(),
    }
}