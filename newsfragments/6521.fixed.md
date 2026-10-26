Fix `with_critical_section_mutex` and `with_critical_section_mutex2` leaving `PyMutex` instances unpoisoned when a panic unwinds out of the closure on free-threaded Python.
