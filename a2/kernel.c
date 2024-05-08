#include <stdint.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>


#include "pcb.h"
#include "kernel.h"
#include "shell.h"
#include "shellmemory.h"
#include "interpreter.h"
#include "ready_queue.h"


#define FRAME_PAGE_SIZE 3 //Each frame is defined as 3 lines long

bool active = false;
bool debug = false;
bool inBackground = false;

//Start a process given a filename that is in the same directory as the executable
int processInitialize(char *filename) {
    //Open the provided file in read mode (in the same directory)
    FILE* sourceFile;
    sourceFile = fopen(filename, "r");

    if (sourceFile == NULL) return FILE_ERROR;

    //Open a target file in read + write mode in the backing store
    int pid = generatePID();
    char targetFileName[256];
    sprintf(targetFileName, "backingStore/%s_%d", filename, pid); // Write file to backing store with filename_pid format

    FILE* targetFile;
    targetFile = fopen(targetFileName, "w+");

    if (targetFile == NULL) return FILE_ERROR;

    //Copy the provided file into the target file
    char readCharacter;
    int numOfLines = 0; //Count the number of lines or "instructions"

    // Copy the contents of the provided file to the target file
    while ((readCharacter = fgetc(sourceFile)) != EOF) {
        if (readCharacter == '\n') numOfLines++; // Count the number of lines or "instructions"

        fputc(readCharacter, targetFile); // Write the character to the target file
    }

    //Close the provided file
    fclose(sourceFile);

    //Pass the file pointer of the target file to shell memory to be loaded
    //We get an array of allocated frames back
    //REWIND THE FILE FOR READING SO YOU DONT SEG FAULT
    rewind(targetFile);
    // Load the first two pages of the file into the frame store, and get the allocated frames indices
    int* allocatedFrames = loadFile(targetFile, numOfLines + 1); //The last line does not end with a \n, therefore +1 to num of lines

    //Create a new PCB with the allocated frames and the number of lines, includes page table instantiation
    PCB* newPCB = makePCB(allocatedFrames, numOfLines + 1, pid, targetFileName); //The last line does not end with a \n, therefore +1 to num of lines
    QueueNode *node = malloc(sizeof(QueueNode));
    node -> pcb = newPCB;

    readyQueueAddToTail(node);
    fclose(targetFile); //Maybe you want to keep this open for next section?
    return 0;
}

// Looping through process execution
bool executeProcess(QueueNode *node, int quanta){
    char *line = NULL;
    PCB *pcb = node->pcb;

    for(int i = 0; i < quanta; i++){
        bool interrupt = false;

        // Skip first incrementation of the process
        if (pcb->instructionsExecuted != 0 ) {
            interrupt = pcb->incrementPC(pcb);
        }

        // Go get the line to be executed
        line = frameGetValueAtLine(pcb->PC);

        // Interrupt due to page fault, process gets placed in the back of the queue
        if (interrupt) {
            return false;
        }

        pcb->instructionsExecuted++;

        inBackground = true;
        if(pcb->priority) {
            pcb->priority = false;
        }

        // Terminate the process if all instructions have been executed
        if(pcb->instructionsExecuted >= (pcb->numOfInstructions)){
            parseInput(line);
            terminateProcess(node);
            inBackground = false;
            return true;
        }

        parseInput(line);
        inBackground = false;

    }
    return false;
}

void *schedulerFCFS() {
    QueueNode *cur;
    bool processComplete = true;

    while (true) {
        if (!processComplete) {
            processComplete = executeProcess(cur, MAX_INT);
            continue;
        }

        if (isReadyEmpty()) {
            if (active) continue;
            else ;break;
        }
        cur = readyQueuePopHead();
        processComplete = executeProcess(cur, MAX_INT);

    }
    return 0;
}

void *schedulerSJF() {
    QueueNode *cur;

    while (true) {
        if (isReadyEmpty()) {
            if (active) continue;
            else break;
        }
        cur = readyQueuePopShortestJob();
        executeProcess(cur, MAX_INT);
    }
    return 0;
}

void *schedulerAGINGAlternative() {
    QueueNode *cur;

    while (true) {
        if (isReadyEmpty()) {
            if (active) continue;
            else break;
        }
        cur = readyQueuePopShortestJob();
        readyQueueDecrementJobLengthScore();
        if (!executeProcess(cur, 1)) {
            readyQueueAddToHead(cur);
        }   
    }
    return 0;
}

void *schedulerAGING() {
    QueueNode *cur;
    int shortest;

    sortReadyQueue();

    while (true) {
        if (isReadyEmpty()) {
            if (active) continue;
            else break;
        }

        cur = readyQueuePopHead();
        shortest = readyQueueGetShortestJobScore();

        if (shortest < cur -> pcb -> jobLengthScore) {
            readyQueuePromote(shortest);
            readyQueueAddToTail(cur);
            cur = readyQueuePopHead();
        }
        readyQueueDecrementJobLengthScore();

        if (!executeProcess(cur, 1)) {
            readyQueueAddToHead(cur);
        }
    }
    return 0;
}

void *schedulerRR(void *arg) {
    int quanta = ((int *) arg)[0];
    QueueNode *cur;

    while (true) {
        // if head is null
        if (isReadyEmpty()) {
            // if process is active continue
            if (active) continue;
            else break;
        }
        // get the head of the queue
        cur = readyQueuePopHead();
        //If execute process is false, put the process at the end of the queue
        //This means a page fault happened, and we should continue rerunning it
        if (!executeProcess(cur, quanta)) {
            readyQueueAddToTail(cur);
        }
    }
    return 0;
}

int scheduleByPolicy(char* policy){
    if (strcmp(policy, "FCFS") != 0 &&
        strcmp(policy, "SJF") != 0 &&
        strcmp(policy, "RR") != 0 &&
        strcmp(policy, "AGING") != 0 &&
        strcmp(policy, "RR30") != 0) {
            return SCHEDULING_ERROR;
    }

    if (active) return 0;
    if (inBackground) return 0;
    int arg[1];

    if (strcmp("FCFS", policy) == 0) {
        schedulerFCFS();
    } else if (strcmp("SJF", policy) == 0) {
        schedulerSJF();
    } else if (strcmp("RR", policy) == 0) {
        arg[0] = 2;
        schedulerRR((void *) arg);
    } else if (strcmp("AGING", policy) == 0) {
        schedulerAGING();
    } else if (strcmp("RR30", policy) == 0) {
        arg[0] = 30;
        schedulerRR((void *) arg);
    }
    return 0;
}


