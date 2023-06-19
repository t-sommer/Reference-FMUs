#ifndef fmi3SharedMemoryFunctions_h
#define fmi3SharedMemoryFunctions_h


#ifdef __cplusplus
extern "C" {
#endif

#include "fmi3Functions.h"

/***************************************************
Shared Memory Functions
****************************************************/

typedef fmi3Status fmi3GetFloat64PointerTYPE(
    fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetFloat64PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64* valuePointers[],
    size_t nValues);

#define fmi3GetFloat64Pointer fmi3FullName(fmi3GetFloat64Pointer)
#define fmi3SetFloat64Pointer fmi3FullName(fmi3SetFloat64Pointer)

FMI3_Export fmi3GetFloat64PointerTYPE fmi3GetFloat64Pointer;
FMI3_Export fmi3SetFloat64PointerTYPE fmi3SetFloat64Pointer;

#ifdef __cplusplus
}  /* end of extern "C" { */
#endif

#endif /* fmi3Functions_h */
