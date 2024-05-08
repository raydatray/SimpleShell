#ifndef SHELLMEMORY_H
#define SHELLMEMORY_H

#include "pcb.h"
void memInit();
char *memGetValue(char *var);
void memSetValue(char *var, char *value);
int* loadFile(FILE* fp, int numOfLines);
int loadPage(FILE* fp, int pageNum);
char* frameGetValueAtLine(int address);
void clearFrame(int frameIndex);
void clearSetOfFrames(int frames[], int numOfFrames);
void printShellMemory();
void resetVarMem();
#endif