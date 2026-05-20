pub mod array;
pub mod boolean;
pub mod decimal;
pub mod integer;
pub mod string;
pub mod table;
pub mod tuple;

//================================================================

use crate::scope::Scope;

//================================================================

pub fn module(scope: &mut Scope) {
    super::primitive::string::module(scope);
    super::primitive::integer::module(scope);
    super::primitive::decimal::module(scope);
    //super::primitive::boolean::module(scope);
    super::primitive::array::module(scope);
    //super::primitive::table::module(scope);
    //super::primitive::tuple::module(scope);
}
