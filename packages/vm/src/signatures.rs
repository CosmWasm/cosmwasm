use wasmer::Type;

#[cfg(test)]
pub const I32_TO_VOID: ([Type; 1], [Type; 0]) = ([Type::I32], []);
#[cfg(test)]
pub const I32_TO_I32: ([Type; 1], [Type; 1]) = ([Type::I32], [Type::I32]);
#[cfg(test)]
pub const I32_I32_TO_VOID: ([Type; 2], [Type; 0]) = ([Type::I32, Type::I32], []);
pub const I32_I32_TO_I32: ([Type; 2], [Type; 1]) = ([Type::I32, Type::I32], [Type::I32]);
#[cfg(test)]
pub const I32_I32_I32_TO_I32: ([Type; 3], [Type; 1]) =
    ([Type::I32, Type::I32, Type::I32], [Type::I32]);
