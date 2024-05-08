#ifndef READY_QUEUE_H
#define READY_QUEUE_H
#include "pcb.h"
#define MAX_INT 2147483646

typedef struct QueueNode{
    PCB *pcb;
    struct QueueNode *next;
} QueueNode;

void readyQueueDestroy();
void readyQueueAddToTail(QueueNode *node);
void printReadyQueue();
void terminateProcess(QueueNode *node);
bool isReadyEmpty();
QueueNode *readyQueuePopShortestJob();
void readyQueueHeadToTail();
void readyQueueAddToHead(QueueNode *node);
QueueNode *readyQueuePopHead();
void readyQueueDecrementJobLengthScore();
void sortReadyQueue();
int readyQueueGetShortestJobScore();
void readyQueuePromote(int score);
#endif