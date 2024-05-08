#include <stdint.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include "pcb.h"
#include "shellmemory.h"

#define FRAME_PAGE_SIZE 3 //Each frame is defined as 3 lines long

int pid_counter = 1;

int generatePID() {
    return pid_counter++;
}

//Increments the PC within the PCB
//Generates an "absolute" 1D address from the number of frames completed and lines completed
//Returns true if a page fault occurs, causing an interrupt
bool incrementPCFunc(PCB* pcb) {
    // If interrupt flag is set then return false, occurs on the execution after a page fault
    if (pcb->interruptFlag) {
        pcb->interruptFlag = false;
        return false;
    }

    //"Roll over" to the next frame if we have completed all the lines within the previous frame
    if (pcb -> instructionsExecuted % FRAME_PAGE_SIZE == 0) {
        pcb -> numberOfFramesExecuted++;
        pcb -> numberOfLinesExecuted = 0;
    } else {
        pcb -> numberOfLinesExecuted ++;
    }

    // Detecting page fault, when the page table entry is -1 represents an unset frame
    if (pcb -> pageTable[pcb -> numberOfFramesExecuted] == -1) {
        //Open the file of the process to load in more pages
        FILE* backingStoreFile;
        backingStoreFile = fopen(pcb->backingStoreName, "r+");

        // Load the page from the backing store and update the page table
        int currentPage = pcb -> instructionsExecuted / FRAME_PAGE_SIZE;
        pcb -> pageTable[pcb -> numberOfFramesExecuted] = loadPage(backingStoreFile,  currentPage + 1);

        // Update the PC pointer
        int newPC = (pcb -> pageTable[pcb -> numberOfFramesExecuted] * FRAME_PAGE_SIZE) + pcb -> numberOfLinesExecuted;

        pcb -> PC = newPC;
        pcb -> interruptFlag = true;

        // Return true, causing an interrupt
        return true;
    }

    // Update the PC pointer
    int newPC = (pcb -> pageTable[pcb -> numberOfFramesExecuted] * FRAME_PAGE_SIZE) + pcb -> numberOfLinesExecuted;
    pcb -> PC = newPC;
    return false;
}


// Create a new PCB with the given parameters
PCB* makePCB(int* allocatedFrames, int numOfInstructions, int pid, char backingStoreName[256]){
    PCB* newPCB = malloc(sizeof(PCB));

    newPCB -> pid = pid; 
    newPCB -> pageTable = allocatedFrames; 
    newPCB -> numOfFrames = (numOfInstructions + (FRAME_PAGE_SIZE - 1)) / FRAME_PAGE_SIZE; //Set number of frames
    newPCB -> PC = newPCB -> pageTable[0] * FRAME_PAGE_SIZE; //Set the PC to the first instruction in the first frame
    newPCB -> numberOfFramesExecuted = 0;
    newPCB -> numberOfLinesExecuted = 0;
    newPCB -> instructionsExecuted = 0;
    newPCB -> numOfInstructions = numOfInstructions;
    newPCB -> jobLengthScore = numOfInstructions; 
    newPCB -> priority = false;
    newPCB -> interruptFlag = false;
    strcpy(newPCB->backingStoreName, backingStoreName);
    newPCB -> incrementPC = &incrementPCFunc;

    return newPCB;
}