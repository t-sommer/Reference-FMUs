#include "fmi3DirectMemory.h"

#include "config.h"
#include "model.h"


void setStartValues(ModelInstance *comp) {

    M(m) = 2;
    M(n) = 2;

    M(u_pointer) = M(u_memory);
    M(y_pointer) = M(y_memory);

    // identity matrix
    for (int i = 0; i < M_MAX; i++) {
    for (int j = 0; j < N_MAX; j++) {
        M(A)[i][j] = i == j ? 1 : 0;
    }}

    for (int i = 0; i < M_MAX; i++) {
        M(u_pointer)[i] = i + 1;
    }

    for (int i = 0; i < N_MAX; i++) {
        M(y_pointer)[i] = 0;
    }

}

Status calculateValues(ModelInstance *comp) {

    // y = A * u
    for (size_t i = 0; i < M(m); i++) {
        M(y_pointer)[i] = 0;
        for (size_t j = 0; j < M(n); j++) {
            M(y_pointer)[i] += M(A)[i][j] * M(u_pointer)[j];
        }
    }

    return OK;
}

Status getFloat64(ModelInstance* comp, ValueReference vr, double values[], size_t nValues, size_t* index) {

    calculateValues(comp);

    switch (vr) {
        case vr_time:
            ASSERT_NVALUES(1);
            values[(*index)++] = comp->time;
            return OK;
        case vr_u:
            ASSERT_NVALUES(M(n));
            for (size_t i = 0; i < M(n); i++) {
                values[(*index)++] = M(u_pointer)[i];
            }
            return OK;
        case vr_A:
            ASSERT_NVALUES(M(m) * M(n));
            for (size_t i = 0; i < M(m); i++)
            for (size_t j = 0; j < M(n); j++) {
                values[(*index)++] = M(A)[i][j];
            }
            return OK;
        case vr_y:
            ASSERT_NVALUES(1);
            for (size_t i = 0; i < M(m); i++) {
                values[(*index)++] = M(y_pointer)[i];
            }
            return OK;
        default:
            logError(comp, "Get Float64 is not allowed for value reference %u.", vr);
            return Error;
    }
}

Status setFloat64(ModelInstance* comp, ValueReference vr, const double values[], size_t nValues, size_t* index) {

    switch (vr) {
        case vr_u:
            ASSERT_NVALUES(M(n));
            for (size_t i = 0; i < M(n); i++) {
                M(u_pointer)[i] = values[(*index)++];
            }
            calculateValues(comp);
            return OK;
        case vr_A:
            ASSERT_NVALUES(M(m) * M(n));
            for (size_t i = 0; i < M(m); i++)
            for (size_t j = 0; j < M(n); j++) {
                M(A)[i][j] = values[(*index)++];
            }
            return OK;
        default:
            logError(comp, "Set Float64 is not allowed for value reference %u.", vr);
            return Error;
    }
}

Status getUInt64(ModelInstance* comp, ValueReference vr, uint64_t values[], size_t nValues, size_t* index) {

    ASSERT_NVALUES(1);

    calculateValues(comp);

    switch (vr) {
        case vr_m:
            values[(*index)++] = M(m);
            return OK;
        case vr_n:
            values[(*index)++] = M(n);
            return OK;
        default:
            logError(comp, "Get UInt64 is not allowed for value reference %u.", vr);
            return Error;
    }
}

Status setUInt64(ModelInstance* comp, ValueReference vr, const uint64_t values[], size_t nValues, size_t* index) {

    ASSERT_NVALUES(1);

    if (comp->state != ConfigurationMode && comp->state != ReconfigurationMode) {
        return Error;
    }

    const uint64_t v = values[(*index)++];

    switch (vr) {
        case vr_m:
            if (v < 1 || v > M_MAX) return Error;
            M(m) = v;
            return OK;
        case vr_n:
            if (v < 1 || v > N_MAX) return Error;
            M(n) = v;
            return OK;
        default:
            logError(comp, "Set UInt64 is not allowed for value reference %u.", vr);
            return Error;
    }
}

void eventUpdate(ModelInstance *comp) {
    comp->valuesOfContinuousStatesChanged   = false;
    comp->nominalsOfContinuousStatesChanged = false;
    comp->terminateSimulation               = false;
    comp->nextEventTimeDefined              = false;
}

fmi3Status fmi3GetFloat64Pointer(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64* valuePointers[],
    size_t nValues) {

    if (!instance) {
        return fmi3Fatal;
    }

    // TODO: check nValues
    UNUSED(nValues);

    ModelInstance* comp = (ModelInstance*)instance;

    calculateValues(comp);

    for (size_t i = 0; i < nValueReferences; i++) {

        const fmi3ValueReference vr = valueReferences[i];

        switch (vr) {
        case vr_u:
            valuePointers[i] = M(u_pointer);
            break;
        case vr_y:
            valuePointers[i] = M(y_pointer);
            break;
        default:
            logError(comp, "fmi3GetFloat64Pointer is not allowed for value reference %u.", vr);
            return fmi3Error;
        }

    }

    return fmi3OK;
}

fmi3Status fmi3SetFloat64Pointer(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64* valuePointers[],
    size_t nValues) {

    if (!instance) {
        return fmi3Fatal;
    }

    // TODO: check nValues
    UNUSED(nValues);

    ModelInstance* comp = (ModelInstance*)instance;

    for (size_t i = 0; i < nValueReferences; i++) {

        const fmi3ValueReference vr = valueReferences[i];

        switch (vr) {
        case vr_u:
            M(u_pointer) = valuePointers[i];
            break;
        case vr_y:
            M(y_pointer) = valuePointers[i];
            break;
        default:
            logError(comp, "fmi3SetFloat64Pointer is not allowed for value reference %u.", vr);
            return fmi3Error;
        }

    }

    return fmi3OK;
}
