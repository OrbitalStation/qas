#define MACRO_1
#define MACRO_3 11

#ifdef MACRO_1
void assume_1() {}
#endif

#ifndef MACRO_2
void assume_2() {}
#endif

#ifndef MACRO_3
doesn't work
#endif

#ifdef MACRO_4
does not work
#endif
