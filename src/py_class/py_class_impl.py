#!/usr/bin/env python

"""
This python script generates the py_class_impl! macro.
"""

from collections import namedtuple
import sys, os

PY2 = (os.getenv('PY') == '2')

header = '''
// Copyright (c) 2016 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
'''

macro_start = '''
#[macro_export]
#[doc(hidden)]
macro_rules! py_class_impl {
    // TT muncher macro. Results are accumulated in $info $slots $impls and $members.
'''

base_case = '''
    // Base case: we're done munching and can start producing code:
    {   {}
        $class:ident $py:ident
        /* info: */ {
            $base_type:ty,
            $size:expr,
            { $( $class_visibility:tt )* },
            $gc:tt,
            /* data: */ [ $( { $data_offset:expr, $data_name:ident, $data_ty:ty } )* ]
        }
        $slots:tt { $( $imp:item )* } $members:tt
    } => {
        py_coerce_item! {
            $($class_visibility)* struct $class { _unsafe_inner: $crate::PyObject }
        }

        py_impl_to_py_object_for_python_object!($class);
        py_impl_from_py_object_for_python_object!($class);

        impl $crate::PythonObject for $class {
            #[inline]
            fn as_object(&self) -> &$crate::PyObject {
                &self._unsafe_inner
            }

            #[inline]
            fn into_object(self) -> $crate::PyObject {
                self._unsafe_inner
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_from(obj: $crate::PyObject) -> Self {
                $class { _unsafe_inner: obj }
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a $crate::PyObject) -> &'a Self {
                ::std::mem::transmute(obj)
            }
        }

        impl $crate::PythonObjectWithCheckedDowncast for $class {
            #[inline]
            fn downcast_from<'p>(py: $crate::Python<'p>, obj: $crate::PyObject) -> Result<$class, $crate::PythonObjectDowncastError<'p>> {
                if py.get_type::<$class>().is_instance(py, &obj) {
                    Ok($class { _unsafe_inner: obj })
                } else {
                    Err($crate::PythonObjectDowncastError(py))
                }
            }

            #[inline]
            fn downcast_borrow_from<'a, 'p>(py: $crate::Python<'p>, obj: &'a $crate::PyObject) -> Result<&'a $class, $crate::PythonObjectDowncastError<'p>> {
                if py.get_type::<$class>().is_instance(py, obj) {
                    unsafe { Ok(::std::mem::transmute(obj)) }
                } else {
                    Err($crate::PythonObjectDowncastError(py))
                }
            }
        }

        py_coerce_item! {
            impl $crate::py_class::BaseObject for $class {
                type InitType = ( $( $data_ty, )* );

                #[inline]
                fn size() -> usize {
                    $size
                }

                unsafe fn alloc(
                    py: $crate::Python,
                    ty: &$crate::PyType,
                    ( $( $data_name, )* ): Self::InitType
                ) -> $crate::PyResult<$crate::PyObject>
                {
                    let obj = try!(<$base_type as $crate::py_class::BaseObject>::alloc(py, ty, ()));
                    $( $crate::py_class::data_init::<$data_ty>(py, &obj, $data_offset, $data_name); )*
                    Ok(obj)
                }

                unsafe fn dealloc(py: $crate::Python, obj: *mut $crate::_detail::ffi::PyObject) {
                    $( $crate::py_class::data_drop::<$data_ty>(py, obj, $data_offset); )*
                    <$base_type as $crate::py_class::BaseObject>::dealloc(py, obj)
                }
            }
        }
        $($imp)*
        py_coerce_item! {
            impl $class {
                fn create_instance(py: $crate::Python $( , $data_name : $data_ty )* ) -> $crate::PyResult<$class> {
                    let obj = try!(unsafe {
                        <$class as $crate::py_class::BaseObject>::alloc(
                            py, &py.get_type::<$class>(), ( $($data_name,)* )
                        )
                    });
                    return Ok($class { _unsafe_inner: obj });

                    // hide statics in create_instance to avoid name conflicts
                    static mut TYPE_OBJECT : $crate::_detail::ffi::PyTypeObject
                        = py_class_type_object_static_init!($class, $gc, $slots);
                    static mut INIT_ACTIVE: bool = false;

                    // trait implementations that need direct access to TYPE_OBJECT
                    impl $crate::PythonObjectWithTypeObject for $class {
                        fn type_object(py: $crate::Python) -> $crate::PyType {
                            unsafe {
                                if $crate::py_class::is_ready(py, &TYPE_OBJECT) {
                                    $crate::PyType::from_type_ptr(py, &mut TYPE_OBJECT)
                                } else {
                                    // automatically initialize the class on-demand
                                    <$class as $crate::py_class::PythonObjectFromPyClassMacro>::initialize(py, None)
                                        .expect(concat!("An error occurred while initializing class ", stringify!($class)))
                                }
                            }
                        }
                    }

                    impl $crate::py_class::PythonObjectFromPyClassMacro for $class {
                        fn initialize(py: $crate::Python, module_name: Option<&str>) -> $crate::PyResult<$crate::PyType> {
                            unsafe {
                                if $crate::py_class::is_ready(py, &TYPE_OBJECT) {
                                    return Ok($crate::PyType::from_type_ptr(py, &mut TYPE_OBJECT));
                                }
                                assert!(!INIT_ACTIVE,
                                    concat!("Reentrancy detected: already initializing class ",
                                    stringify!($class)));
                                INIT_ACTIVE = true;
                                let res = init(py, module_name);
                                INIT_ACTIVE = false;
                                res
                            }
                        }

                        fn add_to_module(py: $crate::Python, module: &$crate::PyModule) -> $crate::PyResult<()> {
                            let ty = <$class as $crate::py_class::PythonObjectFromPyClassMacro>::initialize(py, module.name(py).ok())?;
                            module.add(py, stringify!($class), ty)
                        }
                    }

                    fn init($py: $crate::Python, module_name: Option<&str>) -> $crate::PyResult<$crate::PyType> {
                        py_class_type_object_dynamic_init!($class, $py, TYPE_OBJECT, module_name, $slots);
                        py_class_init_members!($class, $py, TYPE_OBJECT, $members);
                        unsafe {
                            if $crate::_detail::ffi::PyType_Ready(&mut TYPE_OBJECT) == 0 {
                                Ok($crate::PyType::from_type_ptr($py, &mut TYPE_OBJECT))
                            } else {
                                Err($crate::PyErr::fetch($py))
                            }
                        }
                    }
                }
            }
        }
    };
'''

indentation = ['    ']
last_char = '\n'

def write(text):
    global last_char
    for line in text.splitlines(True):
        line = line.lstrip(' ')
        if len(line.strip()) == 0 and last_char == '\n':
            continue
        if last_char == '\n':
            initial_closing = 0
            for c in line:
                if c in ']}':
                    initial_closing += 1
                else:
                    break
            if initial_closing:
                sys.stdout.write(''.join(indentation[:-initial_closing]))
            else:
                sys.stdout.write(''.join(indentation))
        elif last_char not in ' \n' and len(line) > 0 and line[0] not in ' \n;':
            sys.stdout.write(' ')
        sys.stdout.write(line)
        min_indent_level = len(indentation)
        for c in line:
            if c in '[{':
                if len(indentation) > min_indent_level:
                    indentation.append('')
                else:
                    indentation.append('    ')
            elif c in ']}':
                indentation.pop()
                if len(indentation) < min_indent_level:
                    min_indent_level = len(indentation)
        last_char = line[-1]

slot_groups = (
    ('tp', 'type_slots', None),
    ('nb', 'as_number', None),
    ('sq', 'as_sequence', None),
    ('mp', 'as_mapping', None),
    ('sdi', 'setdelitem', ['sdi_setitem', 'sdi_delitem'])
)

def generate_case(pattern, old_info=None, new_info=None, new_impl=None, new_slots=None, new_members=None):
    write('{ { %s $($tail:tt)* }\n' % pattern)
    write('$class:ident $py:ident')
    if old_info is not None:
        write(old_info)
    elif new_info is not None:
        write('\n/* info: */ {\n')
        write('$base_type: ty,\n')
        write('$size: expr,\n')
        write('$class_visibility: tt,\n')
        write('$gc: tt,\n')
        write('[ $( $data:tt )* ]\n')
        write('}\n')
    else:
        write('$info:tt')
    if new_slots:
        write('\n/* slots: */ {\n')
        for prefix, group_name, explicit_slots in slot_groups:
            if any(s.startswith(prefix) for s, v in new_slots):
                if explicit_slots is None:
                    write('\n/* %s */ [ $( $%s_slot_name:ident : $%s_slot_value:expr, )* ]\n'
                          % (group_name, prefix, prefix))
                else:
                    write('\n/* %s */ [\n' % group_name)
                    for slot in explicit_slots:
                        if any(s == slot for s, v in new_slots):
                            write('%s: {},\n' % slot)
                        else:
                            write('%s: $%s_slot_value:tt,\n' % (slot, slot))
                    write(']\n')
            else:
                write('$%s:tt' % group_name)
        write('\n}\n')
    else:
        write('$slots:tt')
    if new_impl is not None:
        write('\n{ $( $imp:item )* }\n')
    else:
        write('$impls:tt')
    if new_members:
        write('\n{ $( $member_name:ident = $member_expr:expr; )* }')
    else:
        write('$members:tt')
    write('\n} => { py_class_impl! {\n')
    write('{ $($tail)* }\n')
    write('$class $py')
    write(new_info or '$info')
    if new_slots:
        write('\n/* slots: */ {\n')
        for prefix, group_name, explicit_slots in slot_groups:
            if any(s.startswith(prefix) for s, v in new_slots):
                write('\n/* %s */ [\n' % group_name)
                if explicit_slots is None:
                    write('$( $%s_slot_name : $%s_slot_value, )*\n' % (prefix, prefix))
                    for s, v in new_slots:
                        if s.startswith(prefix):
                            write('%s: %s,\n' % (s, v))
                else:
                    for slot in explicit_slots:
                        slot_value = next((v for s, v in new_slots if s == slot), None)
                        if slot_value is None:
                            write('%s: $%s_slot_value,\n' % (slot, slot))
                        else:
                            write('%s: { %s },\n' % (slot, slot_value))
                write(']\n')
            else:
                write('$%s' % group_name)
        write('\n}\n')
    else:
        write('$slots')
    if new_impl is not None:
        write('\n/* impl: */ {\n')
        write('$($imp)*\n')
        write(new_impl)
        write('\n}\n')
    else:
        write('$impls')
    if new_members:
        write('\n/* members: */ {\n')
        write('$( $member_name = $member_expr; )*\n')
        for name, val in new_members:
            write('%s = %s;\n' % (name, val))
        write('}')
    else:
        write('$members')
    write('\n}};\n')

def data_decl():
    generate_case('data $data_name:ident : $data_type:ty;',
        new_info = '''
        /* info: */ {
            $base_type,
            /* size: */ $crate::py_class::data_new_size::<$data_type>($size),
            $class_visibility,
            $gc,
            /* data: */ [
                $($data)*
                {
                    $crate::py_class::data_offset::<$data_type>($size),
                    $data_name,
                    $data_type
                }
            ]
        }
        ''',
        new_impl='''
            impl $class {
                fn $data_name<'a>(&'a self, py: $crate::Python<'a>) -> &'a $data_type {
                    unsafe {
                        $crate::py_class::data_get::<$data_type>(
                            py,
                            &self._unsafe_inner,
                            $crate::py_class::data_offset::<$data_type>($size)
                        )
                    }
                }
            }
        ''')

def generate_class_method(special_name=None, decoration='',
        slot=None, add_member=False, value_macro=None, value_args=None):
    name_pattern = special_name or '$name:ident'
    name_use = special_name or '$name'
    def impl(with_params):
        if with_params:
            param_pattern = ', $($p:tt)+'
            impl = '''py_argparse_parse_plist_impl!{
                py_class_impl_item { $class, $py, %s($cls: &$crate::PyType,) $res_type; { $($body)* } }
                [] ($($p)+,)
            }''' % name_use
            value = 'py_argparse_parse_plist_impl!{%s {%s} [] ($($p)+,)}' \
                    % (value_macro, value_args)
        else:
            param_pattern = ''
            impl = 'py_class_impl_item! { $class, $py,%s($cls: &$crate::PyType,) $res_type; { $($body)* } [] }' \
                % name_use
            value = '%s!{%s []}' % (value_macro, value_args)
        pattern = '%s def %s ($cls:ident%s) -> $res_type:ty { $( $body:tt )* }' \
            % (decoration, name_pattern, param_pattern)
        slots = []
        if slot is not None:
            slots.append((slot, value))
        members = []
        if add_member:
            members.append((name_use, value))
        generate_case(pattern, new_impl=impl, new_slots=slots, new_members=members)
    impl(False) # without parameters
    impl(True) # with parameters

def traverse_and_clear():
    generate_case('def __traverse__(&$slf:tt, $visit:ident) $body:block',
        old_info = '''
        /* info: */ {
            $base_type: ty,
            $size: expr,
            $class_visibility: tt,
            /* gc: */ {
                /* traverse_proc: */ None,
                $traverse_data: tt
            },
            $datas: tt
        }
        ''',
        new_info='''
        /* info: */ {
            $base_type,
            $size,
            $class_visibility,
            /* gc: */ {
                /* traverse_proc: */ $class::__traverse__,
                $traverse_data
            },
            $datas
        }
        ''',
        new_impl='''
            py_coerce_item!{
                impl $class {
                    fn __traverse__(&$slf,
                        $py: $crate::Python,
                        $visit: $crate::py_class::gc::VisitProc)
                    -> Result<(), $crate::py_class::gc::TraverseError>
                    $body
                }
            }
        ''')
    generate_case('def __clear__ (&$slf:ident) $body:block',
        new_slots=[('tp_clear', 'py_class_tp_clear!($class)')],
        new_impl='''
            py_coerce_item!{
                impl $class {
                    fn __clear__(&$slf, $py: $crate::Python) $body
                }
            }
        ''')

def generate_instance_method(special_name=None, decoration='',
        slot=None, add_member=False, value_macro=None, value_args=None):
    name_pattern = special_name or '$name:ident'
    name_use = special_name or '$name'
    def impl(with_params):
        if with_params:
            param_pattern = ', $($p:tt)+'
            impl = '''py_argparse_parse_plist_impl!{
                py_class_impl_item { $class, $py, %s(&$slf,) $res_type; { $($body)* } }
                [] ($($p)+,)
            }''' % name_use
            value = 'py_argparse_parse_plist_impl!{%s {%s} [] ($($p)+,)}' \
                    % (value_macro, value_args)
        else:
            param_pattern = ''
            impl = 'py_class_impl_item! { $class, $py, %s(&$slf,) $res_type; { $($body)* } [] }' \
                % name_use
            value = '%s!{%s []}' % (value_macro, value_args)
        pattern = '%s def %s (&$slf:ident%s) -> $res_type:ty { $( $body:tt )* }' \
            % (decoration, name_pattern, param_pattern)
        slots = []
        if slot is not None:
            slots.append((slot, value))
        members = []
        if add_member:
            members.append((name_use, value))
        generate_case(pattern, new_impl=impl, new_slots=slots, new_members=members)
    impl(False) # without parameters
    impl(True) # with parameters

def static_method():
    generate_case(
        '@staticmethod def $name:ident ($($p:tt)*) -> $res_type:ty { $( $body:tt )* }',
        new_impl='''
            py_argparse_parse_plist!{
                py_class_impl_item { $class, $py, $name() $res_type; { $($body)* } }
                ($($p)*)
            }
        ''',
        new_members=[('$name', '''
            py_argparse_parse_plist!{
                py_class_static_method {$py, $class::$name}
                ($($p)*)
            }
        ''')])

def static_data():
    generate_case('static $name:ident = $init:expr;',
        new_members=[('$name', '$init')])

macro_end = '''
}
'''

def special_method(decorated_function):
    def wrap1(*args, **kwargs):
        def wrap2(special_name):
            return decorated_function(special_name, *args, **kwargs)
        return wrap2
    return wrap1

@special_method
def error(special_name, msg):
    print('''
    { { def %s $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "%s" }
    };''' % (special_name, msg))

@special_method
def unimplemented(special_name):
    return error('%s is not supported by py_class! yet.' % special_name)(special_name)

@special_method
def normal_method(special_name):
    pass

@special_method
def special_class_method(special_name, *args, **kwargs):
    generate_class_method(special_name=special_name, *args, **kwargs)

class Argument(object):
    def __init__(self, name):
        self.name = name

@special_method
def operator(special_name, slot,
    args=(),
    res_type='PyObject',
    res_conv=None,
    res_ffi_type='*mut $crate::_detail::ffi::PyObject',
    additional_slots=()
):
    if res_conv is None:
        if res_type == '()':
            res_conv = '$crate::py_class::slots::UnitCallbackConverter'
            res_ffi_type = '$crate::_detail::libc::c_int'
        elif res_type == 'bool':
            res_conv = '$crate::py_class::slots::BoolConverter'
            res_ffi_type = '$crate::_detail::libc::c_int'
        elif res_type == 'PyObject':
            res_conv = '$crate::_detail::PyObjectCallbackConverter'
        else:
            res_conv = '$crate::_detail::PythonObjectCallbackConverter::<$crate::%s>(::std::marker::PhantomData)' % res_type
    arg_pattern = ''
    param_list = []
    for arg in args:
        arg_pattern += ', ${0}:ident : ${0}_type:ty'.format(arg.name)
        param_list.append('{{ ${0} : ${0}_type = {{}} }}'.format(arg.name))
    if slot == 'sq_contains':
        new_slots = [(slot, 'py_class_contains_slot!($class::%s, $%s_type)' % (special_name, args[0].name))]
    elif slot == 'tp_richcompare':
        new_slots = [(slot, 'py_class_richcompare_slot!($class::%s, $%s_type, %s, %s)'
                            % (special_name, args[0].name, res_ffi_type, res_conv))]
    elif len(args) == 0:
        new_slots = [(slot, 'py_class_unary_slot!($class::%s, %s, %s)'
                             % (special_name, res_ffi_type, res_conv))]
    elif len(args) == 1:
        new_slots = [(slot, 'py_class_binary_slot!($class::%s, $%s_type, %s, %s)'
                             % (special_name, args[0].name, res_ffi_type, res_conv))]
    elif len(args) == 2:
        new_slots = [(slot, 'py_class_ternary_slot!($class::%s, $%s_type, $%s_type, %s, %s)'
                             % (special_name, args[0].name, args[1].name, res_ffi_type, res_conv))]
    else:
        raise ValueError('Unsupported argument count')
    generate_case(
        pattern='def %s(&$slf:ident%s) -> $res_type:ty { $($body:tt)* }' % (special_name, arg_pattern),
        new_impl='py_class_impl_item! { $class, $py, %s(&$slf,) $res_type; { $($body)* } [%s] }'
                 % (special_name, ' '.join(param_list)),
        new_slots=new_slots + list(additional_slots)
    )
    # Generate fall-back matcher that produces an error
    # when using the wrong method signature
    error('Invalid signature for operator %s' % special_name)(special_name)

@special_method
def call_operator(special_name, slot):
    generate_instance_method(
        special_name=special_name,
        slot=slot,
        value_macro='py_class_call_slot',
        value_args='$class::%s' % special_name)

@special_method
def binary_numeric_operator(special_name, slot):
    generate_case(
        pattern='def %s($left:ident, $right:ident) -> $res_type:ty { $($body:tt)* }'
            % special_name,
        new_impl='py_class_impl_item! { $class, $py, %s() $res_type; { $($body)* } ' % special_name
                +'[ { $left : &$crate::PyObject = {} } { $right : &$crate::PyObject = {} } ] }',
        new_slots=[(slot, 'py_class_binary_numeric_slot!($class::%s)' % special_name)]
    )
    error('Invalid signature for binary numeric operator %s' % special_name)(special_name)

@special_method
def reflected_numeric_operator(special_name):
    error('Reflected numeric operator %s is not supported by py_class! Use __%s__ instead!'
            % (special_name, special_name[3:-2]))(special_name)

@special_method
def inplace_numeric_operator(special_name, slot):
    operator(slot=slot,
             args=[Argument('other')])(special_name)

special_names = {
    '__init__': error('__init__ is not supported by py_class!; use __new__ instead.'),
    '__new__': special_class_method(
        slot='tp_new',
        value_macro='py_class_wrap_newfunc',
        value_args='$class::__new__'),
    '__del__': error('__del__ is not supported by py_class!; Use a data member with a Drop impl instead.'),
    '__repr__': operator('tp_repr', res_type="PyString"),
    '__str__': operator('tp_str', res_type="PyString"),
    '__unicode__': normal_method(),
    '__bytes__': normal_method(),
    '__format__': normal_method(),
    # Comparison Operators
    '__lt__': error('__lt__ is not supported by py_class! use __richcmp__ instead.'),
    '__le__': error('__le__ is not supported by py_class! use __richcmp__ instead.'),
    '__gt__': error('__gt__ is not supported by py_class! use __richcmp__ instead.'),
    '__ge__': error('__ge__ is not supported by py_class! use __richcmp__ instead.'),
    '__eq__': error('__eq__ is not supported by py_class! use __richcmp__ instead.'),
    '__ne__': error('__ne__ is not supported by py_class! use __richcmp__ instead.'),
    '__cmp__': error('__cmp__ is not supported by py_class! use __richcmp__ instead.'),
    '__richcmp__': operator('tp_richcompare',
        res_type='PyObject',
        args=[Argument('other'), Argument('op')]),
    '__hash__': operator('tp_hash',
        res_conv='$crate::py_class::slots::HashConverter',
        res_ffi_type='$crate::Py_hash_t'),
    '__nonzero__': error('__nonzero__ is not supported by py_class!; use the Python 3 spelling __bool__ instead.'),
    '__bool__': operator('nb_nonzero' if PY2 else 'nb_bool',
        res_type='bool'),
    # Customizing attribute access
    '__getattr__': unimplemented(),
    '__getattribute__': unimplemented(),
    '__setattr__': unimplemented(),
    '__delattr__': unimplemented(),
    '__dir__': unimplemented(),

    # Implementing Descriptors
    '__get__': unimplemented(),
    '__set__': unimplemented(),
    '__delete__': unimplemented(),

    # Customizing instance and subclass checks
    '__instancecheck__': unimplemented(),
    '__subclasscheck__': unimplemented(),

    # Emulating callable objects
    '__call__': call_operator('tp_call'),

    # Emulating container types
    '__len__': operator('sq_length',
                res_ffi_type='$crate::_detail::ffi::Py_ssize_t',
                res_conv='$crate::py_class::slots::LenResultConverter',
                additional_slots=[
                    # Use PySequence_Size to forward mp_length calls to sq_length.
                    ('mp_length', 'Some($crate::_detail::ffi::PySequence_Size)')
                ]),
    '__length_hint__': normal_method(),
    '__getitem__': operator('mp_subscript',
                args=[Argument('key')],
                additional_slots=[
                    ('sq_item', 'Some($crate::py_class::slots::sq_item)')
                ]),
    '__missing__': normal_method(),
    '__setitem__': operator('sdi_setitem',
                args=[Argument('key'), Argument('value')],
                res_type='()'),
    '__delitem__': operator('sdi_delitem',
                args=[Argument('key')],
                res_type='()'),
    '__iter__': operator('tp_iter'),
    '__next__': operator('tp_iternext',
                res_conv='$crate::py_class::slots::IterNextResultConverter'),
    '__reversed__': normal_method(),
    '__contains__': operator('sq_contains', args=[Argument('item')]),

    # Emulating numeric types
    '__add__': binary_numeric_operator('nb_add'),
    '__sub__': binary_numeric_operator('nb_subtract'),
    '__mul__': binary_numeric_operator('nb_multiply'),
    '__matmul__': unimplemented(),
    '__div__': unimplemented(),
    '__truediv__': unimplemented(),
    '__floordiv__': unimplemented(),
    '__mod__': unimplemented(),
    '__divmod__': unimplemented(),
    '__pow__': unimplemented(),
    '__lshift__': binary_numeric_operator('nb_lshift'),
    '__rshift__': binary_numeric_operator('nb_rshift'),
    '__and__': binary_numeric_operator('nb_and'),
    '__xor__': binary_numeric_operator('nb_xor'),
    '__or__': binary_numeric_operator('nb_or'),

    # Emulating numeric types - reflected
    '__radd__': reflected_numeric_operator(),
    '__rsub__': reflected_numeric_operator(),
    '__rmul__': reflected_numeric_operator(),
    '__rmatmul__': reflected_numeric_operator(),
    '__rdiv__': reflected_numeric_operator(),
    '__rtruediv__': reflected_numeric_operator(),
    '__rfloordiv__': reflected_numeric_operator(),
    '__rmod__': reflected_numeric_operator(),
    '__rdivmod__': reflected_numeric_operator(),
    '__rpow__': reflected_numeric_operator(),
    '__rlshift__': reflected_numeric_operator(),
    '__rrshift__': reflected_numeric_operator(),
    '__rand__': reflected_numeric_operator(),
    '__rxor__': reflected_numeric_operator(),
    '__ror__': reflected_numeric_operator(),

    # Emulating numeric types - in-place
    '__iadd__': inplace_numeric_operator('nb_inplace_add'),
    '__isub__': inplace_numeric_operator('nb_inplace_subtract'),
    '__imul__': inplace_numeric_operator('nb_inplace_multiply'),
    '__imatmul__': inplace_numeric_operator('nb_inplace_matrix_multiply'),
    '__idiv__': unimplemented(),
    '__itruediv__': inplace_numeric_operator('nb_inplace_true_divide'),
    '__ifloordiv__': inplace_numeric_operator('nb_inplace_floor_divide'),
    '__imod__': inplace_numeric_operator('nb_inplace_remainder'),
    '__ipow__': unimplemented(),
    '__ilshift__': inplace_numeric_operator('nb_inplace_lshift'),
    '__irshift__': inplace_numeric_operator('nb_inplace_rshift'),
    '__iand__': inplace_numeric_operator('nb_inplace_and'),
    '__ixor__': inplace_numeric_operator('nb_inplace_xor'),
    '__ior__': inplace_numeric_operator('nb_inplace_or'),

    # Unary arithmetic
    '__neg__': operator('nb_negative'),
    '__pos__': operator('nb_positive'),
    '__abs__': operator('nb_absolute'),
    '__invert__': operator('nb_invert'),
    '__complex__': unimplemented(),
    '__int__': unimplemented(),
    '__long__': unimplemented(),
    '__float__': unimplemented(),
    '__round__': unimplemented(),
    '__index__': unimplemented(),
    '__coerce__': unimplemented(),

    # With statement context managers
    '__enter__': normal_method(),
    '__exit__': normal_method(),

    # Coroutines
    '__await__': unimplemented(),
    '__aiter__': unimplemented(),
    '__aenter__': unimplemented(),
    '__aexit__': unimplemented(),
}

def main():
    if sys.argv[1:] == ['--format']:
        while True:
            line = sys.stdin.readline()
            if len(line) == 0:
                return
            write(line)
    print(header)
    print('')
    print('// !!!!!!!!!!!!!!!!!!!!!!!!!!!')
    print('// THIS IS A GENERATED FILE !!')
    print('//       DO NOT MODIFY      !!')
    print('// !!!!!!!!!!!!!!!!!!!!!!!!!!!')
    print(macro_start)
    print(base_case)
    data_decl()
    traverse_and_clear()
    for name, f in sorted(special_names.items()):
        f(name)
    generate_instance_method(
        add_member=True,
        value_macro='py_class_instance_method',
        value_args='$py, $class::$name')
    generate_class_method(decoration='@classmethod',
        add_member=True,
        value_macro='py_class_class_method',
        value_args='$py, $class::$name')
    static_method()
    static_data()
    print(macro_end)

if __name__ == '__main__':
    main()

