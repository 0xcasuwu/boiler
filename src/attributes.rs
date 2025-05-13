// Re-export the attributes from alkanes_runtime and our proc_macros crate
pub use alkanes_runtime::{message::MessageDispatch, runtime::AlkaneResponder};
pub use alkanes_proc_macros::declare_alkane;

// Define attribute types for backward compatibility
#[allow(non_camel_case_types)]
pub struct opcode(pub u32);

#[allow(non_camel_case_types)]
pub struct returns<T>(pub std::marker::PhantomData<T>);

// Type aliases for backward compatibility
pub type OpcodeAttribute = opcode;
pub type ReturnsAttribute<T> = returns<T>;
