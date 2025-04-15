// This header doesn't exist in CPython, but Include/cpython/code.h does. We add
// this here so that PyCodeObject has a definition under the limited API.

opaque_struct!(pub PyCodeObject);
