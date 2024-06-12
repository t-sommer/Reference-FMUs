#pragma once

#include "FMIModelDescription.h"
#include "FMISolver.h"
#include "FMIRecorder.h"


typedef struct {

    // Common
    size_t nStartValues;
    FMIModelVariable** startVariables;
    char** startValues;
    double tolerance;
    double startTime;
    double stopTime;
    double outputInterval;
    const char* initialFMUStateFile;
    const char* finalFMUStateFile;

    // Recorder
    FMIRecorder* recorder;
    FMIRecorderSample sample;

    // Co-Simulation
    bool earlyReturnAllowed;
    bool eventModeUsed;
    bool recordIntermediateValues;

    // Model Exchange
    SolverCreate solverCreate;
    SolverFree solverFree;
    SolverStep solverStep;
    SolverReset solverReset;

} FMISimulationSettings;

FMIStatus FMIApplyStartValues(FMIInstance* S, const FMISimulationSettings* settings);
