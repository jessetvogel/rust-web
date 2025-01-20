use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    // Buffer used to communicate from JS to Rust. Initial size is 1024.
    // Grows when `get_allocation` is called from JS with size bigger than current size.
    static ALLOCATION: RefCell<Vec<u8>> = RefCell::new(vec![0; 1024]);
}

extern "C" {
    fn __invoke(c_ptr: *const u8, c_len: u32, p_ptr: *const u8, p_len: u32);
    fn __free_object(id: u32);
    fn __create_element(t_ptr: *const u8, t_len: u32);
    fn __query_selector(q_ptr: *const u8, q_len: u32);
}

#[no_mangle]
pub fn get_allocation(size: usize) -> *const u8 {
    ALLOCATION.with(|rc| {
        let mut vec = rc.borrow_mut();
        if vec.len() < size {
            *vec = vec![0; size];
        }
        vec.as_ptr()
    })
}

// JavaScript objects that are returned by __invoke are represented by an id of type u32.
// To make sure JavaScript frees the object when Rust no longer needs it, the id is uniquely
// owned by an ObjectId. When this object is dropped, a signal is sent to JavaScript to free the object.
#[derive(Debug)]
pub struct ObjectId(u32);

impl Drop for ObjectId {
    fn drop(&mut self) {
        unsafe {
            __free_object(self.0);
        }
    }
}

// An ObjectRef contains a shared reference to an ObjectId.
// This way, there can be multiple owners of the JavaScript object,
// and when everyone has dropped the reference, the object will be dropped automatically.
#[derive(Debug, Clone)]
pub struct ObjectRef(Rc<ObjectId>);

impl ObjectRef {
    pub fn new(id: u32) -> Self {
        Self(Rc::new(ObjectId(id)))
    }

    pub fn id(&self) -> u32 {
        self.0 .0
    }
}

// NOTE: Numbers in Javascript are represented by 64-bits floats
// https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types-number-type
#[derive(Debug)]
pub enum JsValue {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    BigInt(i64),
    String(String),
    Object(ObjectRef),
    Blob(Vec<u8>),
    Array(ObjectRef),
}

pub fn invoke(code: &str, params: &[JsValue]) -> JsValue {
    let code = __code(code, params);
    let params = serialize(params);

    unsafe {
        __invoke(
            code.as_ptr(),
            code.len() as u32,
            params.as_ptr(),
            params.len() as u32,
        )
    };

    let values = ALLOCATION
        .with_borrow(|buffer| deserialize(buffer))
        .expect("invalid response from JS");

    values.into_iter().next().unwrap_or(JsValue::Undefined)
}

pub fn query_selector(query: &str) -> JsValue {
    unsafe { __query_selector(query.as_ptr(), query.len() as u32) };

    let values = ALLOCATION
        .with_borrow(|buffer| deserialize(buffer))
        .expect("invalid response from JS");

    values.into_iter().next().unwrap_or(JsValue::Undefined)
}

pub fn create_element(tag: &str) -> JsValue {
    unsafe { __create_element(tag.as_ptr(), tag.len() as u32) };

    let values = ALLOCATION
        .with_borrow(|buffer| deserialize(buffer))
        .expect("invalid response from JS");

    values.into_iter().next().unwrap_or(JsValue::Undefined)
}

fn serialize(values: &[JsValue]) -> Vec<u8> {
    let mut buffer = Vec::new();

    buffer.extend(u32::to_le_bytes(values.len() as u32));

    for value in values {
        match value {
            JsValue::Undefined => buffer.push(0x00),
            JsValue::Null => buffer.push(0x01),
            JsValue::Bool(true) => buffer.push(0x02),
            JsValue::Bool(false) => buffer.push(0x03),
            JsValue::Number(f) => {
                buffer.push(0x04);
                buffer.extend(f64::to_le_bytes(*f));
            }
            JsValue::BigInt(i) => {
                buffer.push(0x05);
                buffer.extend(i64::to_le_bytes(*i));
            }
            JsValue::String(s) => {
                buffer.push(0x06);
                buffer.extend(u32::to_le_bytes(s.len() as u32));
                buffer.extend(s.as_bytes());
            }
            JsValue::Array(r) => {
                buffer.push(0x07);
                buffer.extend(u32::to_le_bytes(r.id()));
            }
            JsValue::Object(r) => {
                buffer.push(0x08);
                buffer.extend(u32::to_le_bytes(r.id()));
            }
            JsValue::Blob(v) => {
                buffer.push(0x09);
                buffer.extend(u32::to_le_bytes(v.len() as u32));
                buffer.extend(v);
            }
        }
    }

    buffer
}

fn deserialize(buffer: &[u8]) -> Result<Vec<JsValue>, &'static str> {
    let mut values = Vec::new();

    let len = u32::from_le_bytes(buffer[0..4].try_into().unwrap()) as usize;

    let mut i = 4;

    for _ in 0..len {
        let (size, value) = match buffer[i] {
            0x00 => (1, JsValue::Undefined),
            0x01 => (1, JsValue::Null),
            0x02 => (1, JsValue::Bool(true)),
            0x03 => (1, JsValue::Bool(false)),
            0x04 => (
                9,
                JsValue::Number(f64::from_le_bytes(buffer[i + 1..i + 9].try_into().unwrap())),
            ),
            0x05 => (
                9,
                JsValue::BigInt(i64::from_le_bytes(buffer[i + 1..i + 9].try_into().unwrap())),
            ),
            0x06 => {
                let len = u32::from_le_bytes(buffer[i + 1..i + 5].try_into().unwrap()) as usize;
                let str = String::from_utf8_lossy(&buffer[i + 5..i + 5 + len]);
                (5 + len, JsValue::String(str.into()))
            }
            0x07 => (
                5,
                JsValue::Array(ObjectRef::new(u32::from_le_bytes(
                    buffer[i + 1..i + 5].try_into().unwrap(),
                ))),
            ),
            0x08 => (
                5,
                JsValue::Object(ObjectRef::new(u32::from_le_bytes(
                    buffer[i + 1..i + 5].try_into().unwrap(),
                ))),
            ),
            0x09 => {
                let len = u32::from_le_bytes(buffer[i + 1..i + 5].try_into().unwrap()) as usize;
                (5 + len, JsValue::Blob(buffer[i + 5..i + 5 + len].to_vec()))
            }
            _ => return Err("invalid type"),
        };
        i += size;
        values.push(value);
    }

    Ok(values)
}

fn __code(code: &str, params: &[JsValue]) -> String {
    let mut code_params = String::from(code);

    let params_names = params
        .iter()
        .enumerate()
        .map(|(i, _)| "p".to_owned() + &i.to_string())
        .collect::<Vec<_>>();
    for param_name in &params_names {
        if let Some(pos) = code_params.find("{}") {
            code_params.replace_range(pos..pos + 2, param_name);
        }
    }
    format!("function({}){{{}}}", params_names.join(","), code_params)
}

impl From<&str> for JsValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}
impl From<String> for JsValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}
impl From<f64> for JsValue {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}
impl From<f32> for JsValue {
    fn from(n: f32) -> Self {
        Self::Number(n as f64)
    }
}
impl From<u32> for JsValue {
    fn from(n: u32) -> Self {
        Self::Number(n as f64)
    }
}
impl From<u64> for JsValue {
    fn from(n: u64) -> Self {
        Self::Number(n as f64)
    }
}
impl From<i32> for JsValue {
    fn from(n: i32) -> Self {
        Self::Number(n as f64)
    }
}
impl From<i64> for JsValue {
    fn from(n: i64) -> Self {
        Self::Number(n as f64)
    }
}
impl From<bool> for JsValue {
    fn from(n: bool) -> Self {
        Self::Bool(n)
    }
}
impl From<ObjectRef> for JsValue {
    fn from(s: ObjectRef) -> Self {
        Self::Object(s)
    }
}
impl From<&ObjectRef> for JsValue {
    fn from(s: &ObjectRef) -> Self {
        Self::Object(s.to_owned())
    }
}
impl From<Vec<u8>> for JsValue {
    fn from(s: Vec<u8>) -> Self {
        Self::Blob(s)
    }
}

impl JsValue {
    pub fn to_bool(self) -> Result<bool, &'static str> {
        match self {
            JsValue::Bool(b) => Ok(b),
            _ => Err("invalid type"),
        }
    }

    pub fn to_string(self) -> Result<String, &'static str> {
        match self {
            JsValue::String(s) => Ok(s),
            _ => Err("invalid type"),
        }
    }

    pub fn to_num(self) -> Result<f64, &'static str> {
        match self {
            JsValue::Number(s) => Ok(s),
            _ => Err("invalid type"),
        }
    }
    pub fn to_ref(self) -> Result<ObjectRef, String> {
        match self {
            JsValue::Object(s) => Ok(s),
            _ => Err(format!("invalid type ({:?})", self)),
        }
    }
    pub fn to_buffer(self) -> Result<Vec<u8>, &'static str> {
        match self {
            JsValue::Blob(s) => Ok(s),
            _ => Err("invalid type"),
        }
    }

    pub fn to_bigint(self) -> Result<i64, &'static str> {
        match self {
            JsValue::BigInt(s) => Ok(s),
            _ => Err("invalid type"),
        }
    }
}
