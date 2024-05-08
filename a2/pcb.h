#ifndef PCB_H
#define PCB_H
#include <stdbool.h>

/*
 * Struct:  PCB 
 * --------------------
 * pid: process(task) id
 * PC: program counter, stores the index of line that the task is executing
 * start: the first line in shell memory that belongs to this task
 * end: the last line in shell memory that belongs to this task
 * job_length_score: for EXEC AGING use only, stores the job length score
 */
typedef struct PCB{
    int pid;
    int PC;
    bool priority;
    int numberOfFramesExecuted;
    int numberOfLinesExecuted;
    int instructionsExecuted;
    int numOfInstructions;
    int jobLengthScore;
    int numOfFrames;
    int pageTableSize;
    bool interruptFlag;

    bool (*incrementPC)(struct PCB *pcb);

    int* pageTable; //This has to be at the bottom

    char backingStoreName[256];
} PCB;

int generatePID();
PCB* makePCB(int* allocatedFrames, int numOfInstructions, int pid, char backingStoreName[256]);
#endif