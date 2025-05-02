use syn::{parse_quote, File};

pub(crate) fn get_fixed_serialization() -> File {
    parse_quote! {

        use solana_program::log::{sol_log, sol_log_data};
        use std::any::type_name_of_val;
        use std::cell::RefCell;
        use std::rc::Rc;
        use solana_program::account_info::AccountInfo;
        use solana_program::pubkey::Pubkey;

        pub trait _SolanaDebuggerSerialize {
            fn _solana_debugger_serialize(&self, name: &str);
        }

        impl<T: ?Sized> _SolanaDebuggerSerialize for T {
            default fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("not_implemented");
                sol_log("END_NODE");
            }
        }

        macro_rules! impl_serialize {
            ($type:ty, $is_complex:expr, $ser_type:expr, $data_ser:expr) => {
                impl _SolanaDebuggerSerialize for $type {
                    fn _solana_debugger_serialize(&self, name: &str) {
                        sol_log("START_NODE");
                        sol_log(if $is_complex { "complex" } else { "primitive" });
                        sol_log(name);
                        sol_log(type_name_of_val(self));
                        sol_log($ser_type);
                        ($data_ser)(self);
                        sol_log("END_NODE");
                    }
                }
            }
        }

        macro_rules! impl_serialize_int {
            ($type:ty) => {
                impl_serialize!(
                    $type,
                    false,
                    "int",
                    |s: &$type| {
                        let bytes = (*s as i128).to_le_bytes();
                        sol_log_data(&[bytes.as_slice()]);
                    }
                );
            }
        }

        impl_serialize_int!(i8);
        impl_serialize_int!(i16);
        impl_serialize_int!(i32);
        impl_serialize_int!(i64);
        impl_serialize_int!(i128);

        macro_rules! impl_serialize_uint {
            ($type:ty) => {
                impl_serialize!(
                    $type,
                    false,
                    "uint",
                    |s: &$type| {
                        let bytes = (*s as u128).to_le_bytes();
                        sol_log_data(&[bytes.as_slice()]);
                    }
                );
            }
        }

        impl_serialize_uint!(u8);
        impl_serialize_uint!(u16);
        impl_serialize_uint!(u32);
        impl_serialize_uint!(u64);
        impl_serialize_uint!(u128);

        impl _SolanaDebuggerSerialize for bool {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("primitive");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("bool");
                sol_log_data(&[&[*self as u8]]);
                sol_log("END_NODE");
            }
        }

        impl _SolanaDebuggerSerialize for &str {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("primitive");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("str");
                sol_log(self);
                sol_log("END_NODE");
            }
        }

        impl _SolanaDebuggerSerialize for String {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("primitive");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("str");
                sol_log(self.as_str());
                sol_log("END_NODE");
            }
        }

        impl<T> _SolanaDebuggerSerialize for Option<T> {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("str_ident");
                let variant_str = match self {
                    None => "None",
                    Some(_) => "Some"
                };
                sol_log(variant_str);
                if let Some(v) = self {
                    v._solana_debugger_serialize("0");
                }
                sol_log("END_NODE");
            }
        }

        impl<T, E> _SolanaDebuggerSerialize for Result<T, E> {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("str_ident");
                let variant_str = match self {
                    Ok(_) => "Ok",
                    Err(_) => "Err"
                };
                sol_log(variant_str);
                match self {
                    Ok(v) => {
                        v._solana_debugger_serialize("0");
                    },
                    Err(v) => {
                        v._solana_debugger_serialize("0");
                    }
                }
                sol_log("END_NODE");
            }
        }

        impl<T1, T2> _SolanaDebuggerSerialize for (T1, T2) {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("no_data");
                self.0._solana_debugger_serialize("0");
                self.1._solana_debugger_serialize("1");
                sol_log("END_NODE");
            }
        }

        impl<T> _SolanaDebuggerSerialize for Box<T> {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("no_data");
                (**self)._solana_debugger_serialize("value");
                sol_log("END_NODE");
            }
        }

        impl<T> _SolanaDebuggerSerialize for Rc<T> {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("rc_meta");
                let strong_count = (Rc::strong_count(self) as u128).to_le_bytes();
                sol_log_data(&[strong_count.as_slice()]);
                let weak_count = (Rc::weak_count(self) as u128).to_le_bytes();
                sol_log_data(&[weak_count.as_slice()]);
                (**self)._solana_debugger_serialize("value");
                sol_log("END_NODE");
            }
        }

        impl<T> _SolanaDebuggerSerialize for RefCell<T> {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));

                if let Ok(v) = self.try_borrow() {
                    sol_log("no_data");
                    (*v)._solana_debugger_serialize("value");
                } else {
                    sol_log("error_str");
                    sol_log("Failed to borrow");
                }

                sol_log("END_NODE");
            }
        }

        impl<T> _SolanaDebuggerSerialize for Vec<T> {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));

                sol_log("array_len");

                let len = (self.len() as u128).to_le_bytes();
                sol_log_data(&[len.as_slice()]);

                for el in self {
                    el._solana_debugger_serialize("-inc-index");
                }

                sol_log("END_NODE");
            }
        }

        impl<T> _SolanaDebuggerSerialize for &[T] {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));

                sol_log("array_len");

                let len = (self.len() as u128).to_le_bytes();
                sol_log_data(&[len.as_slice()]);

                for el in (*self).iter() {
                    el._solana_debugger_serialize("-inc-index");
                }

                sol_log("END_NODE");
            }
        }

        impl<T> _SolanaDebuggerSerialize for &mut [T] {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));

                sol_log("array_len");

                let len = (self.len() as u128).to_le_bytes();
                sol_log_data(&[len.as_slice()]);

                for el in (*self).iter() {
                    el._solana_debugger_serialize("-inc-index");
                }

                sol_log("END_NODE");
            }
        }
        impl<T: Sized> _SolanaDebuggerSerialize for &T {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("no_data");

                (**self)._solana_debugger_serialize("value");

                sol_log("END_NODE");
            }
        }

        impl<T> _SolanaDebuggerSerialize for &mut T {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("no_data");

                (**self)._solana_debugger_serialize("value");

                sol_log("END_NODE");
            }
        }

        impl<T, const N: usize> _SolanaDebuggerSerialize for [T; N] {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));

                sol_log("array_len");

                let len = (N as u128).to_le_bytes();
                sol_log_data(&[len.as_slice()]);

                for el in self {
                    el._solana_debugger_serialize("-inc-index");
                }

                sol_log("END_NODE");
            }
        }

        impl<'a> _SolanaDebuggerSerialize for AccountInfo<'a> {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));
                sol_log("no_data");

                self.key._solana_debugger_serialize("key");
                self.lamports._solana_debugger_serialize("lamports");
                self.data._solana_debugger_serialize("data");
                self.owner._solana_debugger_serialize("owner");
                self.rent_epoch._solana_debugger_serialize("rent_epoch");
                self.is_signer._solana_debugger_serialize("is_signer");
                self.is_writable._solana_debugger_serialize("is_writable");
                self.executable._solana_debugger_serialize("executable");

                sol_log("END_NODE");
            }
        }

        impl _SolanaDebuggerSerialize for Pubkey {
            fn _solana_debugger_serialize(&self, name: &str) {
                sol_log("START_NODE");
                sol_log("complex");
                sol_log(name);
                sol_log(type_name_of_val(self));

                sol_log("pubkey");

                sol_log_data(&[self.as_ref()]);

                sol_log("END_NODE");
            }
        }
    }
}
