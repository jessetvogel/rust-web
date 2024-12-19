
use std::ops::Deref;

#[cfg(not(test))]
extern "C" {
    fn __invoke(c_ptr: *const u8, c_len: u32, p_ptr: *const u8, p_len: u32) -> u64;
    fn __deallocate(object_id: *const u8);
}

#[cfg(test)]
unsafe fn __invoke(_c_ptr: *const u8, _c_len: u32, _p_ptr: *const u8, _p_len: u32) -> u64 { 0 }
#[cfg(test)]
unsafe fn __deallocate(_object_id: *const u8) {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectRef(u32);

impl ObjectRef {
    pub fn new(object_id: u32) -> Self {
        Self (object_id)
    }
}

impl Deref for ObjectRef {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// NOTE: Numbers in Javascript are represented by 64-bits floats
// https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types-number-type
#[derive(Debug)]
pub enum JsValue {
    Undefined,
    Null,
    BigInt(i64),
    Str(String),
    Bool(bool),
    Number(f64),
    Buffer(Vec<u8>),
    Ref(ObjectRef),
}

pub use JsValue::*;

impl JsValue {

    // layout: type (1 byte) - data (var length)
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Undefined => vec![0],
            Null => vec![1],
            Buffer(b) => [vec![2], b.to_owned()].concat(),
            BigInt(i) => [vec![3], i.to_le_bytes().to_vec()].concat(),
            Str(s) => [vec![4], (s.as_ptr() as u32).to_le_bytes().to_vec(), s.len().to_le_bytes().to_vec()].concat(),
            Bool(b) => vec![if *b { 5 } else { 6 }],
            Ref(i) => [vec![7], i.0.to_le_bytes().to_vec()].concat(),
            Number(i) => [vec![8], i.to_le_bytes().to_vec()].concat(),
        }
    }

    pub fn deserialize(r_type: u32, r_value: u32) -> Self {
        match r_type {
            0 => JsValue::Undefined,
            1 => JsValue::Number(r_value as f64),
            2 => JsValue::Ref(ObjectRef(r_value)),
            3 => {
                let allocation_data = crate::allocations::ALLOCATIONS.with_borrow_mut(|s| s.remove(r_value as usize));
                JsValue::Str(String::from_utf8_lossy(&allocation_data).into())
            },
            4 => {
                JsValue::Buffer(crate::allocations::ALLOCATIONS.with_borrow_mut(|s| s.remove(r_value as usize)))
            },
            5 => JsValue::Bool(if r_value == 1 { true } else { false }),

            _ => unreachable!(),
        }
    }

    pub fn to_bool(&self) -> Result<bool, String> {
        match &self {
            JsValue::Bool(b) => Ok(b.to_owned()),
            _ => Err("Invalid type".to_string()),
        }
    }

    pub fn to_str(&self) -> Result<String, String> {
        match &self {
            JsValue::Str(s) => Ok(s.to_string()),
            _ => Err("Invalid type".to_string()),
        }
    }

    pub fn to_num(&self) -> Result<f64, String> {
        match &self {
            JsValue::Number(s) => Ok(s.to_owned()),
            _ => Err("Invalid type".to_string()),
        }
    }
    pub fn to_ref(&self) -> Result<ObjectRef, String> {
        match &self {
            JsValue::Ref(s) => Ok(s.to_owned()),
            _ => Err("Invalid type".to_string()),
        }
    }
    pub fn to_buffer(&self) -> Result<Vec<u8>, String> {
        match &self {
            JsValue::Buffer(s) => Ok(s.to_owned()),
            _ => Err("Invalid type".to_string()),
        }
    }
}

#[derive(Debug)]
pub enum ReturnParam { Void = 0, Number = 1, Ref = 2, Str = 3, Buffer = 4 }


pub struct Js {}

impl Js {
    fn __code(code: &str, params: &[JsValue]) -> String {

        let mut code_params = String::from(code);

        let params_names = params.iter().enumerate().map(|(i, _)| "p".to_owned() + &i.to_string()).collect::<Vec<_>>();
        for param_name in &params_names {
            if let Some(pos) = code_params.find("{}") {
                code_params.replace_range(pos..pos + 2, param_name);
            }
        }
        format!("function({}) {{ {} }}", params_names.join(","), code_params)
    }
    pub fn invoke<'a>(code: &'a str, params: &[JsValue]) -> JsValue {
        let code = Self::__code(code, params);
        let params = params.iter().flat_map(JsValue::serialize).collect::<Vec<_>>();
        let r_packed = unsafe { __invoke(code.as_ptr(), code.len() as u32, params.as_ptr(), params.len() as u32) };
        let r_type = (r_packed >> 32) as u32;
        let r_value = (r_packed & 0xFFFFFFFF) as u32;
        JsValue::deserialize(r_type, r_value)
    }
    pub fn deallocate(object_id: ObjectRef) {
        unsafe { __deallocate(*object_id as *const u8) };
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn cs(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect::<String>()
    }

    #[test]
    fn test_params() {

        // undefined
        assert_eq!(Undefined.serialize(), vec![0]);

        // null
        assert_eq!(Null.serialize(), vec![1]);

        // bigint
        assert_eq!(BigInt(42).serialize(), [vec![3], 42u64.to_le_bytes().to_vec()].concat());

        // string
        let text = "hello".to_owned();
        let text_ptr = text.as_ptr() as u32;
        let text_len = text.len() as u64;
        let expected = [vec![4], text_ptr.to_le_bytes().to_vec(), text_len.to_le_bytes().to_vec()].concat();
        assert_eq!(Str(text).serialize(), expected);

        // bool
        assert_eq!(Bool(true).serialize(), vec![5]);
        assert_eq!(Bool(false).serialize(), vec![6]);

        // object ref
        assert_eq!(Ref(ObjectRef(42)).serialize(), [vec![7], 42u32.to_le_bytes().to_vec()].concat());

        // uint
        assert_eq!(Number(42.into()).serialize(), [vec![8], 42f64.to_le_bytes().to_vec()].concat());

    }

    #[test]
    fn test_code() {
        // prompt
        let code = Js::__code("return prompt({},{})", &[Str("a".into()), Str("b".into())]);
        let expected_code = "function(p0,p1){ return prompt(p0,p1) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // console log
        let code = Js::__code("console.log({})", &[Str("a".into())]);
        let expected_code = "function(p0){ console.log(p0) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // alert
        let code = Js::__code("alert({})", &[Str("a".into())]);
        let expected_code = "function(p0){ alert(p0) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // set attribute
        let code = Js::__code("{}.setAttribute({},{})", &[Ref(ObjectRef(0)), Str("a".into()), Str("b".into())]);
        let expected_code = "function(p0,p1,p2){ p0.setAttribute(p1, p2) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // append child
        let code = Js::__code("{}.appendChild({})", &[Ref(ObjectRef(0)), Ref(ObjectRef(0))]);
        let expected_code = "function(p0,p1){ p0.appendChild(p1) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // add class
        let code = Js::__code("{}.classList.add({})", &[Ref(ObjectRef(0)), Str("a".into())]);
        let expected_code = "function(p0,p1){ p0.classList.add(p1) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // set property
        let code = Js::__code("{}[{}] = {}", &[Ref(ObjectRef(0)), Str("a".into()), Str("a".into())]);
        let expected_code = "function(p0,p1,p2){ p0[p1] = p2 }";
        assert_eq!(cs(&code), cs(&expected_code));

        // set inner html
        let code = Js::__code("{}.innerHTML = {}", &[Ref(ObjectRef(0)), Str("a".into())]);
        let expected_code = "function(p0,p1){ p0.innerHTML = p1 }";
        assert_eq!(cs(&code), cs(&expected_code));

        // history push state
        // NOTE: {} is parsed as the first parameter
        let code = Js::__code("window.history.pushState({ },{},{})", &[Str("a".into()), Str("b".into())]);
        let expected_code = "function(p0,p1){ window.history.pushState({ },p0,p1) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // location pathname
        let code = Js::__code("return window.location.pathname", &[]);
        let expected_code = "function() { return window.location.pathname }";
        assert_eq!(cs(&code), cs(&expected_code));

        // get property string
        let code = Js::__code("return {}[{}]", &[Ref(ObjectRef(0)), Str("b".into())]);
        let expected_code = "function(p0,p1){ return p0[p1] }";
        assert_eq!(cs(&code), cs(&expected_code));

        // prompt dialog
        let code = Js::__code("return prompt({},{})", &[Str("a".into()), Str("b".into())]);
        let expected_code = "function(p0,p1){ return prompt(p0,p1) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // random number
        let code = Js::__code("return Math.random()", &[]);
        let expected_code = "function(){ return Math.random() }";
        assert_eq!(cs(&code), cs(&expected_code));

        // get property
        let code = Js::__code("return {}[{}]", &[Ref(ObjectRef(0)), Str("a".into())]);
        let expected_code = "function(p0,p1){ return p0[p1] }";
        assert_eq!(cs(&code), cs(&expected_code));

        // query selector
        let code = Js::__code("return document.querySelector({})", &[Str("a".into())]);
        let expected_code = "function(p0){ return document.querySelector(p0) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // create element
        let code = Js::__code("return document.createElement({})", &[Str("a".into())]);
        let expected_code = "function(p0){ return document.createElement(p0) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // create text node
        let code = Js::__code("return document.createTextNode({})", &[Str("a".into())]);
        let expected_code = "function(p0){ return document.createTextNode(p0) }";
        assert_eq!(cs(&code), cs(&expected_code));

    }
}
