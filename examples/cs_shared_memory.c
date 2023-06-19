#define OUTPUT_FILE  "cs_shared_memory_out.csv"
#define LOG_FILE     "cs_shared_memory_log.txt"

#include "util.h"


int main(int argc, char* argv[]) {

    CALL(setUp());

    CALL(FMI3InstantiateCoSimulation(S,
        INSTANTIATION_TOKEN, // instantiationToken
        NULL,                // resourcePath
        fmi3False,           // visible
        fmi3False,           // loggingOn
        fmi3False,           // eventModeUsed
        fmi3True,            // earlyReturnAllowed
        NULL,                // requiredIntermediateVariables
        0,                   // nRequiredIntermediateVariables
        NULL                 // intermediateUpdate
    ));

    fmi3Float64 time = startTime;

    fmi3Float64 u[2] = { 0, 0 };

    fmi3ValueReference input_vrs[1] = { vr_u };
    fmi3Float64* input_pointers[1] = { u };

    CALL(FMI3SetFloat64Pointer(S, input_vrs, 1, input_pointers, 2));

    // initialize the FMU
    CALL(FMI3EnterInitializationMode(S, fmi3False, 0.0, time, fmi3True, stopTime));

    CALL(FMI3ExitInitializationMode(S));

    // communication step size
    const fmi3Float64 stepSize = 2;

    while (true) {

        // use the importer's memory to provide the inputs
        u[0] = -time; u[1] = time;
        CALL(FMI3SetFloat64Pointer(S, input_vrs, 1, input_pointers, 2));

        // record variables
        CALL(recordVariables(S, outputFile));

        if (terminateSimulation || time >= stopTime) {
            break;
        }

        CALL(FMI3DoStep(S,
            time,                 // currentCommunicationPoint
            stepSize,             // communicationStepSize
            fmi3True,             // noSetFMUStatePriorToCurrentPoint
            &eventEncountered,    // eventEncountered
            &terminateSimulation, // terminate
            &earlyReturn,         // earlyReturn
            &time                 // lastSuccessfulTime
        ));
    }

TERMINATE:
    return tearDown();
}
