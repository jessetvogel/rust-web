use crate::element::Elem;

pub trait Component {
    fn to_elem(&self) -> &Elem;
}
