#ifndef fmi3SharedMemoryFunctions_h
#define fmi3SharedMemoryFunctions_h


#ifdef __cplusplus
extern "C" {
#endif

#include "fmi3Functions.h"

/***************************************************
Direct Memory Functions
****************************************************/

typedef fmi3Status fmi3GetFloat32PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float32* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetFloat64PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetInt8PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Int8* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetUInt8PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3UInt8* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetInt16PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Int16* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetUInt16PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3UInt16* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetInt32PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Int32* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetUInt32PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3UInt32* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetInt64PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Int64* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetUInt64PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3UInt64* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetBooleanPointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Boolean* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetStringPointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3String valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetBinaryPointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    size_t valueSizes[],
    fmi3Binary valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3GetClockPointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Clock* valuePointers[]);

typedef fmi3Status fmi3SetFloat32PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float32* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetFloat64PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetInt8PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Int8* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetUInt8PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3UInt8* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetInt16PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Int16* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetUInt16PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3UInt16* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetInt32PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Int32* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetUInt32PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3UInt32* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetInt64PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Int64* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetUInt64PointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3UInt64* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetBooleanPointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Boolean* valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetStringPointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3String valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetBinaryPointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    const size_t valueSizes[],
    fmi3Binary valuePointers[],
    size_t nValues);

typedef fmi3Status fmi3SetClockPointerTYPE(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Clock* valuePointers[]);

#define fmi3GetFloat32Pointer fmi3FullName(fmi3GetFloat32Pointer)
#define fmi3GetFloat64Pointer fmi3FullName(fmi3GetFloat64Pointer)
#define fmi3GetInt8Pointer    fmi3FullName(fmi3GetInt8Pointer)
#define fmi3GetUInt8Pointer   fmi3FullName(fmi3GetUInt8Pointer)
#define fmi3GetInt16Pointer   fmi3FullName(fmi3GetInt16Pointer)
#define fmi3GetUInt16Pointer  fmi3FullName(fmi3GetUInt16Pointer)
#define fmi3GetInt32Pointer   fmi3FullName(fmi3GetInt32Pointer)
#define fmi3GetUInt32Pointer  fmi3FullName(fmi3GetUInt32Pointer)
#define fmi3GetInt64Pointer   fmi3FullName(fmi3GetInt64Pointer)
#define fmi3GetUInt64Pointer  fmi3FullName(fmi3GetUInt64Pointer)
#define fmi3GetBooleanPointer fmi3FullName(fmi3GetBooleanPointer)
#define fmi3GetStringPointer  fmi3FullName(fmi3GetStringPointer)
#define fmi3GetBinaryPointer  fmi3FullName(fmi3GetBinaryPointer)
#define fmi3GetClockPointer   fmi3FullName(fmi3GetClockPointer)
#define fmi3SetFloat32Pointer fmi3FullName(fmi3SetFloat32Pointer)
#define fmi3SetFloat64Pointer fmi3FullName(fmi3SetFloat64Pointer)
#define fmi3SetInt8Pointer    fmi3FullName(fmi3SetInt8Pointer)
#define fmi3SetUInt8Pointer   fmi3FullName(fmi3SetUInt8Pointer)
#define fmi3SetInt16Pointer   fmi3FullName(fmi3SetInt16Pointer)
#define fmi3SetUInt16Pointer  fmi3FullName(fmi3SetUInt16Pointer)
#define fmi3SetInt32Pointer   fmi3FullName(fmi3SetInt32Pointer)
#define fmi3SetUInt32Pointer  fmi3FullName(fmi3SetUInt32Pointer)
#define fmi3SetInt64Pointer   fmi3FullName(fmi3SetInt64Pointer)
#define fmi3SetUInt64Pointer  fmi3FullName(fmi3SetUInt64Pointer)
#define fmi3SetBooleanPointer fmi3FullName(fmi3SetBooleanPointer)
#define fmi3SetStringPointer  fmi3FullName(fmi3SetStringPointer)
#define fmi3SetBinaryPointer  fmi3FullName(fmi3SetBinaryPointer)
#define fmi3SetClockPointer   fmi3FullName(fmi3SetClockPointer)

FMI3_Export fmi3GetFloat32PointerTYPE fmi3GetFloat32Pointer;
FMI3_Export fmi3GetFloat64PointerTYPE fmi3GetFloat64Pointer;
FMI3_Export fmi3GetInt8PointerTYPE    fmi3GetInt8Pointer;
FMI3_Export fmi3GetUInt8PointerTYPE   fmi3GetUInt8Pointer;
FMI3_Export fmi3GetInt16PointerTYPE   fmi3GetInt16Pointer;
FMI3_Export fmi3GetUInt16PointerTYPE  fmi3GetUInt16Pointer;
FMI3_Export fmi3GetInt32PointerTYPE   fmi3GetInt32Pointer;
FMI3_Export fmi3GetUInt32PointerTYPE  fmi3GetUInt32Pointer;
FMI3_Export fmi3GetInt64PointerTYPE   fmi3GetInt64Pointer;
FMI3_Export fmi3GetUInt64PointerTYPE  fmi3GetUInt64Pointer;
FMI3_Export fmi3GetBooleanPointerTYPE fmi3GetBooleanPointer;
FMI3_Export fmi3GetStringPointerTYPE  fmi3GetStringPointer;
FMI3_Export fmi3GetBinaryPointerTYPE  fmi3GetBinaryPointer;
FMI3_Export fmi3GetClockPointerTYPE   fmi3GetClockPointer;
FMI3_Export fmi3SetFloat32PointerTYPE fmi3SetFloat32Pointer;
FMI3_Export fmi3SetFloat64PointerTYPE fmi3SetFloat64Pointer;
FMI3_Export fmi3SetInt8PointerTYPE    fmi3SetInt8Pointer;
FMI3_Export fmi3SetUInt8PointerTYPE   fmi3SetUInt8Pointer;
FMI3_Export fmi3SetInt16PointerTYPE   fmi3SetInt16Pointer;
FMI3_Export fmi3SetUInt16PointerTYPE  fmi3SetUInt16Pointer;
FMI3_Export fmi3SetInt32PointerTYPE   fmi3SetInt32Pointer;
FMI3_Export fmi3SetUInt32PointerTYPE  fmi3SetUInt32Pointer;
FMI3_Export fmi3SetInt64PointerTYPE   fmi3SetInt64Pointer;
FMI3_Export fmi3SetUInt64PointerTYPE  fmi3SetUInt64Pointer;
FMI3_Export fmi3SetBooleanPointerTYPE fmi3SetBooleanPointer;
FMI3_Export fmi3SetStringPointerTYPE  fmi3SetStringPointer;
FMI3_Export fmi3SetBinaryPointerTYPE  fmi3SetBinaryPointer;
FMI3_Export fmi3SetClockPointerTYPE   fmi3SetClockPointer;

///////////////


//typedef fmi3Status fmi3GetFloat64PointerTYPE(
//    fmi3Instance instance,
//    const fmi3ValueReference valueReferences[],
//    size_t nValueReferences,
//    fmi3Float64* valuePointers[],
//    size_t nValues);
//
//typedef fmi3Status fmi3SetFloat64PointerTYPE(fmi3Instance instance,
//    const fmi3ValueReference valueReferences[],
//    size_t nValueReferences,
//    fmi3Float64* valuePointers[],
//    size_t nValues);
//
//#define fmi3GetFloat64Pointer fmi3FullName(fmi3GetFloat64Pointer)
//#define fmi3SetFloat64Pointer fmi3FullName(fmi3SetFloat64Pointer)
//
//FMI3_Export fmi3GetFloat64PointerTYPE fmi3GetFloat64Pointer;
//FMI3_Export fmi3SetFloat64PointerTYPE fmi3SetFloat64Pointer;

#ifdef __cplusplus
}  /* end of extern "C" { */
#endif

#endif /* fmi3Functions_h */
