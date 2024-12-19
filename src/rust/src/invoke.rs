
use std::ops::Deref;

#[cfg(not(test))]
extern "C" {
    fn __invoke_and_return(c_ptr: *const u8, c_len: u32, p_ptr: *const u8, p_len: u32, r_type: u8) -> u32;
    fn __deallocate(object_id: *const u8);
}

#[cfg(test)]
unsafe fn __invoke_and_return(_c_ptr: *const u8, _c_len: u32, _p_ptr: *const u8, _p_len: u32, _r_type: u8) -> u32 { 0 }
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
pub enum InvokeParam<'a> {
    Undefined,
    Null,
    BigInt(i64),
    Str(&'a str),
    Bool(bool),
    Number(f64),
    Ref(&'a ObjectRef),
}

pub use InvokeParam::*;

impl<'a> InvokeParam<'a> {

    // layout: type (1 byte) - data (var length)
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Undefined => vec![0],
            Null => vec![1],
            BigInt(i) => [vec![3], i.to_le_bytes().to_vec()].concat(),
            Str(s) => [vec![4], (s.as_ptr() as u32).to_le_bytes().to_vec(), s.len().to_le_bytes().to_vec()].concat(),
            Bool(b) => vec![if *b { 5 } else { 6 }],
            Ref(i) => [vec![7], i.0.to_le_bytes().to_vec()].concat(),
            Number(i) => [vec![8], i.to_le_bytes().to_vec()].concat(),
        }
    }
}

#[derive(Debug)]
pub enum ReturnParam { Void = 0, Number = 1, Ref = 2, Str = 3, Buffer = 4 }


pub struct Js {}

impl Js {
    fn __code(code: &str, params: &[InvokeParam]) -> String {

        let mut code_params = String::from(code);

        let params_names = params.iter().enumerate().map(|(i, _)| "p".to_owned() + &i.to_string()).collect::<Vec<_>>();
        for param_name in &params_names {
            if let Some(pos) = code_params.find("{}") {
                code_params.replace_range(pos..pos + 2, param_name);
            }
        }
        format!("function({}) {{ {} }}", params_names.join(","), code_params)
    }
    fn __invoke(code: &str, params: &[InvokeParam], r_type: ReturnParam) -> u32 {
        let code = Self::__code(code, params);
        let params = params.iter().flat_map(InvokeParam::serialize).collect::<Vec<_>>();
        unsafe { __invoke_and_return(code.as_ptr(), code.len() as u32, params.as_ptr(), params.len() as u32, r_type as u8) }
    }
    pub fn invoke(code: &str, params: &[InvokeParam]) {
        Self::__invoke(code, params, ReturnParam::Void);
    }
    pub fn invoke_str(code: &str, params: &[InvokeParam]) -> String {
        let allocation_id = Self::__invoke(code, params, ReturnParam::Str);
        let allocation_data = crate::allocations::ALLOCATIONS.with_borrow_mut(|s| s.remove(allocation_id as usize));
        String::from_utf8(allocation_data).unwrap()
    }
    pub fn invoke_number(code: &str, params: &[InvokeParam]) -> u32 {
        Self::__invoke(code, params, ReturnParam::Number)
    }
    pub fn invoke_ref(code: &str, params: &[InvokeParam]) -> ObjectRef {
        let object_ref = Self::__invoke(code, params, ReturnParam::Ref);
        ObjectRef(object_ref)
    }
    pub fn invoke_buffer(code: &str, params: &[InvokeParam]) -> Vec<u8> {
        let allocation_id = Self::__invoke(code, params, ReturnParam::Buffer);
        crate::allocations::ALLOCATIONS.with_borrow_mut(|s| s.remove(allocation_id as usize))
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
        let text = "hello";
        let text_ptr = text.as_ptr() as u32;
        let text_len = text.len() as u64;
        let expected = [vec![4], text_ptr.to_le_bytes().to_vec(), text_len.to_le_bytes().to_vec()].concat();
        assert_eq!(Str(text).serialize(), expected);

        // bool
        assert_eq!(Bool(true).serialize(), vec![5]);
        assert_eq!(Bool(false).serialize(), vec![6]);

        // object ref
        assert_eq!(Ref(&ObjectRef(42)).serialize(), [vec![7], 42u32.to_le_bytes().to_vec()].concat());

        // uint
        assert_eq!(Number(42.into()).serialize(), [vec![8], 42f64.to_le_bytes().to_vec()].concat());

    }

    #[test]
    fn test_code() {
        // prompt
        let code = Js::__code("return prompt({},{})", &[Str("a"), Str("b")]);
        let expected_code = "function(p0,p1){ return prompt(p0,p1) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // console log
        let code = Js::__code("console.log({})", &[Str("a")]);
        let expected_code = "function(p0){ console.log(p0) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // alert
        let code = Js::__code("alert({})", &[Str("a")]);
        let expected_code = "function(p0){ alert(p0) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // set attribute
        let code = Js::__code("{}.setAttribute({},{})", &[Ref(&ObjectRef(0)), Str("a"), Str("b")]);
        let expected_code = "function(p0,p1,p2){ p0.setAttribute(p1, p2) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // append child
        let code = Js::__code("{}.appendChild({})", &[Ref(&ObjectRef(0)), Ref(&ObjectRef(0))]);
        let expected_code = "function(p0,p1){ p0.appendChild(p1) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // add class
        let code = Js::__code("{}.classList.add({})", &[Ref(&ObjectRef(0)), Str("a")]);
        let expected_code = "function(p0,p1){ p0.classList.add(p1) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // set property
        let code = Js::__code("{}[{}] = {}", &[Ref(&ObjectRef(0)), Str("a"), Str("a")]);
        let expected_code = "function(p0,p1,p2){ p0[p1] = p2 }";
        assert_eq!(cs(&code), cs(&expected_code));

        // set inner html
        let code = Js::__code("{}.innerHTML = {}", &[Ref(&ObjectRef(0)), Str("a")]);
        let expected_code = "function(p0,p1){ p0.innerHTML = p1 }";
        assert_eq!(cs(&code), cs(&expected_code));

        // history push state
        // NOTE: {} is parsed as the first parameter
        let code = Js::__code("window.history.pushState({ },{},{})", &[Str("a"), Str("b")]);
        let expected_code = "function(p0,p1){ window.history.pushState({ },p0,p1) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // location pathname
        let code = Js::__code("return window.location.pathname", &[]);
        let expected_code = "function() { return window.location.pathname }";
        assert_eq!(cs(&code), cs(&expected_code));

        // get property string
        let code = Js::__code("return {}[{}]", &[Ref(&ObjectRef(0)), Str("b")]);
        let expected_code = "function(p0,p1){ return p0[p1] }";
        assert_eq!(cs(&code), cs(&expected_code));

        // prompt dialog
        let code = Js::__code("return prompt({},{})", &[Str("a"), Str("b")]);
        let expected_code = "function(p0,p1){ return prompt(p0,p1) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // random number
        let code = Js::__code("return Math.random()", &[]);
        let expected_code = "function(){ return Math.random() }";
        assert_eq!(cs(&code), cs(&expected_code));

        // get property
        let code = Js::__code("return {}[{}]", &[Ref(&ObjectRef(0)), Str("a")]);
        let expected_code = "function(p0,p1){ return p0[p1] }";
        assert_eq!(cs(&code), cs(&expected_code));

        // query selector
        let code = Js::__code("return document.querySelector({})", &[Str("a")]);
        let expected_code = "function(p0){ return document.querySelector(p0) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // create element
        let code = Js::__code("return document.createElement({})", &[Str("a")]);
        let expected_code = "function(p0){ return document.createElement(p0) }";
        assert_eq!(cs(&code), cs(&expected_code));

        // create text node
        let code = Js::__code("return document.createTextNode({})", &[Str("a")]);
        let expected_code = "function(p0){ return document.createTextNode(p0) }";
        assert_eq!(cs(&code), cs(&expected_code));

    }

    #[test]
    fn test_invoke() {

        // invoke
        let result = Js::invoke_number("", &[]);
        assert_eq!(result, 0);

        // invoke and return object
        let result = Js::invoke_ref("", &[]);
        assert_eq!(result, ObjectRef(0));

        // invoke and return string
        let text = "hello";
        crate::allocations::ALLOCATIONS.with_borrow_mut(|s| {
            *s = vec![text.as_bytes().to_vec()];
        });
        let result = Js::invoke_str("", &[]);
        assert_eq!(result, "hello".to_owned());

        // invoke and return array buffer
        let vec = vec![1, 2];
        crate::allocations::ALLOCATIONS.with_borrow_mut(|s| {
            *s = vec![vec.clone()];
        });
        let result = Js::invoke_buffer("", &[]);
        assert_eq!(result, vec);

    }
}
