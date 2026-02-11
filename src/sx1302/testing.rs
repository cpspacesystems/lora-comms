use core::str;
use std::{any::TypeId, cell::UnsafeCell, collections::HashMap, fmt::Debug, marker::PhantomData, ops::Not};

use paste::paste;

/// Marks what kind of parameter a functions's arguments are 
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParamType<T> {
    /// Respresents an arugment that is only read by the function as an input
    /// <br> This is asserted against what is passed into the function everytime the harness is called. 
    Input{ input: T },
    /// Respresents an argument or return that is only written to by the function as an output
    Output{ output: T },
    /// Respresents an argument that is both read by the function and written to
    /// <br> The input value is asserted against what is passed into the function everytime the harness is called. 
    InputOutput{ input: T, output: T },
    /// Respresents an argument or return that is nothing
    Nothing,
    /// Respresents an argument that exists, but is bypassed during checking
    Bypassed,
}

/// Respresents an argument or return that is nothing
/// This is the same as ParamType<T>::Nothing. 
/// This ZST exists only to allow for variadic like behavior in rust using default generics  
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Nothing;


impl<T> ParamType<T>  {
    /// gets a refernce for function input, None if this ParamType does not support input
    pub fn get_read_ref(&self) -> Option<&T> {
        match self {
            ParamType::Input{ input: v} => Some(v),
            ParamType::Output{ output: _ } => None,
            ParamType::InputOutput{ input: v, output: _ } => Some(v),
            ParamType::Nothing => None,
            ParamType::Bypassed => None,
        } 
    }
    /// gets a reference for a function output, None if this ParamType does not support output
    /// <br> Note: clone the ouputted data to write otherwise you get lifetime issues
    pub fn get_write_ref(&self) -> Option<&T> {
        match self {
            ParamType::Input{ input:  _ } => None,
            ParamType::Output{ output: v} => Some(v),
            ParamType::InputOutput{ input: _, output: v} => Some(v),
            ParamType::Nothing => None,
            ParamType::Bypassed => None,
        }
    }
}

/// Respresents the state of a function
/// <br> Create this using the macro `new_FunctionData!{ ... }`
#[derive(Debug, Clone, Copy)]
pub struct FunctionState<
     R: PartialEq + Clone, 
    A1: PartialEq + Clone, 
    A2: PartialEq + Clone,
    A3: PartialEq + Clone,
    A4: PartialEq + Clone,
    A5: PartialEq + Clone,
>{
    pub next_return: ParamType<R>,
    pub arg1: ParamType<A1>,
    pub arg2: ParamType<A2>,
    pub arg3: ParamType<A3>,
    pub arg4: ParamType<A4>,
    pub arg5: ParamType<A5>,
}

/// internal helper macro for parsing new_FunctionData's ParamTypes fields
/// <br> This is exported to allow for other macros to be used, do not use this directly
#[allow(unused_macros, reason = "Some Intellisense linters accidently mark this as unused.")]
macro_rules! helper_resolve_type_of_arg {
    (r $r:expr) => { crate::sx1302::testing::ParamType::Input{ input: {$r} } };
    (w $w:expr) => { crate::sx1302::testing::ParamType::Output{ output: {$w} } };
    (rw $r:expr; $w:expr) => { crate::sx1302::testing::ParamType::InputOutput { input: {$r}, output: {$w} } };    
    (b $b:expr) => { crate::sx1302::testing::ParamType::Bypassed };
}
pub(super) use helper_resolve_type_of_arg as helper_resolve_type_of_arg;

/// internal helper macro for parsing new_FunctionData's ParamTypes fields
/// <br> This is exported to allow for other macros to be used, do not use this directly
#[allow(unused_macros, reason = "Some Intellisense linters accidently mark this as unused.")]
macro_rules! helper_resolve_value_or_Nothing {
    ($i:ident $v:expr $(;$v2:expr)?) => { crate::sx1302::testing::helper_resolve_type_of_arg!{ $i $v $(;$v2)? } };
    () => { crate::sx1302::testing::ParamType::Nothing };
}
pub(super) use helper_resolve_value_or_Nothing as helper_resolve_value_or_Nothing;

/// Creates a new FunctionData with a specified amount of return and argument values<br> 
/// This is a workaround for rust not having variadtic arguments
/// 
/// Syntax: `<target param> <access type> <value> [; <optional value 2>]?` <br>
/// `<target param>: ret | arg1 | arg2 | arg3 | arg4 | arg5` Return value and positional function arguments <br> 
/// `<access type>: r | w | rw | b ` <br>
/// &emsp; `r` Value is only read and used as an input. As assert will be done to ensure FunctionData's value and the actual function call actually receive the same thing<br>
/// &emsp; `w` Value is only written to and used as an output, the related value in FunctionData will be returned by writing to that pointer or reference<br>
/// &emsp; `rw` Value is bot read and written to. An assert will be done using value and value2 will be returned<br>
/// &emsp; `b` This argument is by passed. You do also need to set value to Bypassed <br>
/// `<value>` The first value, can be anything. This is used for all operations<br>
/// `; <optional value 2>` The second value, only used for specifying the output of `rw`.<br>
/// 
/// Example: 
/// ```
/// new_FunctionData! {
///     ret w <return value>, 
///     arg1 r <function pos 1 param value>,
///     arg2 w <function pos 2 param value>,
///     arg3 rw <function pos 3 param value>,
///     arg4 b Bypassed,
///     ... // obmit arg3 to arg5 if your func only takes 2 parameters 
///     arg5 r/w/rw <function pos 5 param value>
/// }
/// ```
/// Or you can just do `new_FunctionData! {}` to get an FunctionData with no return and no arguments
#[allow(unused_macros, reason = "Some Intellisense linters accidently mark this as unused.")]
macro_rules! new_FunctionData {
    (
        $(ret w $ret_val:expr)?
        $(, arg1 $arg1_t:ident $arg1_val:expr $(; $arg1_val2:expr)?)?
        $(, arg2 $arg2_t:ident $arg2_val:expr $(; $arg2_val2:expr)?)?
        $(, arg3 $arg3_t:ident $arg3_val:expr $(; $arg3_val2:expr)?)?
        $(, arg4 $arg4_t:ident $arg4_val:expr $(; $arg4_val2:expr)?)?
        $(, arg5 $arg5_t:ident $arg5_val:expr $(; $arg5_val2:expr)?)?
        $(,)?
    ) => {
        crate::sx1302::testing::FunctionState {
            next_return: crate::sx1302::testing::helper_resolve_value_or_Nothing!($(w $ret_val)?),
            arg1: crate::sx1302::testing::helper_resolve_value_or_Nothing!($($arg1_t $arg1_val $(; $arg1_val2)? )?),
            arg2: crate::sx1302::testing::helper_resolve_value_or_Nothing!($($arg2_t $arg2_val $(; $arg2_val2)? )?),
            arg3: crate::sx1302::testing::helper_resolve_value_or_Nothing!($($arg3_t $arg3_val $(; $arg3_val2)? )?),
            arg4: crate::sx1302::testing::helper_resolve_value_or_Nothing!($($arg4_t $arg4_val $(; $arg4_val2)? )?),
            arg5: crate::sx1302::testing::helper_resolve_value_or_Nothing!($($arg5_t $arg5_val $(; $arg5_val2)? )?),
        }
    };
}
pub(super) use new_FunctionData as new_FunctionData;

/// This macro has the exact same signature and usuage as new_FuctionData.
/// This is here just as syntax sugar for creating TestHarness for harness implenmenters.
/// <br> This is the same as: `TestHarness::new(new_FunctionData!{ ... })` 
#[allow(unused_macros, reason = "Some Intellisense linters accidently mark this as unused.")]
macro_rules! new_TestHarness {
    (
        $(ret w $ret_val:expr)?
        $(, arg1 $arg1_t:ident $arg1_val:expr $(; $arg1_val2:expr)?)?
        $(, arg2 $arg2_t:ident $arg2_val:expr $(; $arg2_val2:expr)?)?
        $(, arg3 $arg3_t:ident $arg3_val:expr $(; $arg3_val2:expr)?)?
        $(, arg4 $arg4_t:ident $arg4_val:expr $(; $arg4_val2:expr)?)?
        $(, arg5 $arg5_t:ident $arg5_val:expr $(; $arg5_val2:expr)?)?
        $(,)?
    ) => {
        crate::sx1302::testing::TestHarness::new(crate::sx1302::testing::FunctionState {
            next_return: crate::sx1302::testing::helper_resolve_value_or_Nothing!($(w $ret_val)?),
            arg1: crate::sx1302::testing::helper_resolve_value_or_Nothing!($($arg1_t $arg1_val $(; $arg1_val2)? )?),
            arg2: crate::sx1302::testing::helper_resolve_value_or_Nothing!($($arg2_t $arg2_val $(; $arg2_val2)? )?),
            arg3: crate::sx1302::testing::helper_resolve_value_or_Nothing!($($arg3_t $arg3_val $(; $arg3_val2)? )?),
            arg4: crate::sx1302::testing::helper_resolve_value_or_Nothing!($($arg4_t $arg4_val $(; $arg4_val2)? )?),
            arg5: crate::sx1302::testing::helper_resolve_value_or_Nothing!($($arg5_t $arg5_val $(; $arg5_val2)? )?),
        })
    };
}
pub(super) use new_TestHarness as new_TestHarness;

/// This macro allows for variadtic expansion of TestHarness.call_harness_hook. 
/// This is just syntax sugar for harness implenmneters so we don't need to write all the arguments even for those that are Nothing.
/// <br> Defination: `call_harness_hook!(self: TestHarness, arg1_value: &mut Arg1Type, arg2_value: &mut Arg2Type, ..., arg5_value: &mut Arg5Type)`
/// <br>
/// <br> Ex: `call_harness_hook!(self, v1, v2); // expandss to => self.call_harness_hook(&mut v1, &mut v2, &mut Nothing, &mut Nothing, &mut Nothing);`
#[allow(unused_macros, reason = "Some Intellisense linters accidently mark this as unused.")]
macro_rules! call_harness_hook {
    ($self:expr) => { $self.call_harness_hook(&mut Nothing, &mut Nothing, &mut Nothing, &mut Nothing, &mut Nothing) };
    ($self:expr, $a1:expr) => { $self.call_harness_hook($a1, &mut Nothing, &mut Nothing, &mut Nothing, &mut Nothing) };
    ($self:expr, $a1:expr, $a2:expr) => { $self.call_harness_hook($a1, $a2, &mut Nothing, &mut Nothing, &mut Nothing) };
    ($self:expr, $a1:expr, $a2:expr, $a3:expr) => { $self.call_harness_hook($a1, $a2, $a3, &mut Nothing, &mut Nothing) };
    ($self:expr, $a1:expr, $a2:expr, $a3:expr,  $a4:expr) => { $self.call_harness_hook($a1, $a2, $a3, $a4, &mut Nothing) };
    ($self:expr, $a1:expr, $a2:expr, $a3:expr,  $a4:expr, $a5:expr) => { $self.call_harness_hook($a1, $a2, $a3, $a4, $a5) };
}
pub(super) use call_harness_hook as call_harness_hook; 

/// TestHarness for integration tests / mocking
/// <br> you can conviently define a TestHarness using `new_TestHarness!{ ... }` macro
pub struct TestHarness<
     R: Debug + PartialEq + Clone = Nothing, 
    A1: Debug + PartialEq + Clone = Nothing, 
    A2: Debug + PartialEq + Clone = Nothing,
    A3: Debug + PartialEq + Clone = Nothing,
    A4: Debug + PartialEq + Clone = Nothing,
    A5: Debug + PartialEq + Clone = Nothing,
> {
    call_count: usize,
    typical_output: FunctionState<R, A1, A2, A3, A4, A5>,
    // maps call_count -> expected output at that frame
    // if no match, no asserts are done and everything is bypassed
    expectation_map: HashMap<usize, FunctionState<R, A1, A2, A3, A4, A5>>
}

impl<R: Debug + PartialEq + Clone, 
    A1: Debug + PartialEq + Clone, 
    A2: Debug + PartialEq + Clone,
    A3: Debug + PartialEq + Clone,
    A4: Debug + PartialEq + Clone,
    A5: Debug + PartialEq + Clone,
> TestHarness<R, A1, A2, A3, A4, A5> {
    /// Creates a new TestHarness
    /// <br> Recommended to use `new_TestHarness` macro unless you want to write 10 lines to create one harness
    pub fn new(typical_output: FunctionState<R, A1, A2, A3, A4, A5>) -> Self {        
        TestHarness {
            call_count: 0,
            typical_output,
            expectation_map: HashMap::new(),
        }
    }

    /// Expects the function harnessed to be at some state on some number of calls from now
    pub fn expect_from_now(&mut self, calls_from_now: usize, expected_state: FunctionState<R, A1, A2, A3, A4, A5>) -> &mut Self {
        self.expect_on_call(self.call_count + calls_from_now, expected_state)
    }
    /// Expects the function harnessed to be at some state after it has been called some number of times
    pub fn expect_on_call(&mut self, call_count: usize, expected_state: FunctionState<R, A1, A2, A3, A4, A5>) -> &mut Self{
        self.expectation_map.insert(call_count, expected_state);
        self
    }
    /// Sets the typical output of the harnessed function.
    pub fn set_typical_output(&mut self, typical_output: FunctionState<R, A1, A2, A3, A4, A5>) -> &mut Self {
        self.typical_output = typical_output;
        self
    }
    /// Removes all expectations. This could be useful if you are running out of memory after setting a lot of expectations.
    pub fn remove_all_expects(&mut self) -> &mut Self {
        self.expectation_map.clear();
        self
    }
    /// Resets the internal call counter back to 0 
    pub fn reset_call_count(&mut self) -> &mut Self {
        self.call_count = 0;
        self
    }
    /// Calls the harness hook. The function harnessed should call this method 
    /// and pass it all arguments (with some processing if needed); 
    /// while also returning the value returned by call_harness_hook.
    /// <br>
    /// See `call_harness_hook!()` macro for convient way to call this method.
    pub fn call_harness_hook(&mut self, arg1: &mut A1, arg2: &mut A2, arg3: &mut A3, arg4: &mut A4, arg5: &mut A5) -> Option<R> {
        self.call_count += 1;
        let working_fdata = match self.expectation_map.get(&self.call_count) {
            Some(expectation) => expectation,
            None => &self.typical_output
        };
        Self::handle_arg(&working_fdata.arg1, arg1);
        Self::handle_arg(&working_fdata.arg2, arg2);
        Self::handle_arg(&working_fdata.arg3, arg3);
        Self::handle_arg(&working_fdata.arg4, arg4);
        Self::handle_arg(&working_fdata.arg5, arg5);

        working_fdata.next_return.get_write_ref().cloned()
    }
    // handling checking asserts for a function argument 
    fn handle_arg<T: Debug + PartialEq + Clone>(expected_arg: &ParamType<T>, arg: &mut T) {
        if let Some(v) = expected_arg.get_read_ref() {
            assert_eq!(*v, *arg);
        }

        if let Some(v) = expected_arg.get_write_ref() {
            *arg = v.clone();
        }
    }
}


