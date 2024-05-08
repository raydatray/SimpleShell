#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>
#include "ready_queue.h"

QueueNode *head = NULL;

void readyQueueDestroy() {
    if (!head) return;
    QueueNode *cur = head;
    QueueNode *tmp;
    while (cur -> next != NULL) {
        tmp = cur -> next;
        free(cur);
        cur = tmp;
    }
    free(cur);
}

void readyQueueAddToTail(QueueNode *node) {
    if (!head) {
        head = node;
        head -> next = NULL;
    } else {
        QueueNode *cur = head;
        while (cur -> next != NULL) cur = cur -> next;
        cur -> next = node;
        cur -> next -> next = NULL;
    }
}

void readyQueueAddToHead(QueueNode *node) {
    if (!head) {
        head = node;
        head -> next = NULL;
    } else {
        node -> next = head;
        head = node;
    }
}

void printReadyQueue() {
    if (!head) {
        printf("ready queue is empty\n");
        return;
    }
    QueueNode *cur = head;
    printf("Ready queue: \n");
    while (cur != NULL) {
        cur = cur -> next;
    }
}

void terminateProcess(QueueNode *node) {
    // Node should not be in the ready queue
    free(node -> pcb -> pageTable);
    free(node -> pcb);
    free(node);
}

bool isReadyEmpty() {
    return head==NULL;
}

QueueNode *readyQueuePopHead(){
    QueueNode *tmp = head;
    if (head != NULL) head = head -> next;
    return tmp;
}

void readyQueueDecrementJobLengthScore() {
    QueueNode *cur;
    cur = head;
    while (cur != NULL) {
        if (cur -> pcb -> jobLengthScore > 0) cur -> pcb -> jobLengthScore--;
        cur = cur -> next;
    }
}

void readyQueueSwapWithNext(QueueNode *toSwap) {
    QueueNode *next;
    QueueNode *afterNext;
    QueueNode *cur = head;
    if (head == toSwap) {
        next = head -> next;
        head -> next = next -> next;
        next -> next = head;
        head = next;
    }
    while (cur != NULL && cur -> next != toSwap) cur = cur -> next;
    if (cur == NULL) return;
    next = cur -> next -> next;
    afterNext = next -> next;
    //cur toSwap next afterNext
    cur -> next = next;
    next -> next = toSwap;
    toSwap -> next = afterNext;
}

bool swapNeeded(QueueNode *cur){
    QueueNode *next = cur -> next;
    if (!next) return false;
    if (cur -> pcb -> priority && next -> pcb -> priority) {
        if (cur -> pcb -> jobLengthScore > next -> pcb -> jobLengthScore) {
            return true;
        } else {
            return false;
        }
    } else if (cur -> pcb -> priority && !next -> pcb -> priority) {
        return false;
    } else if (!cur -> pcb -> priority && next -> pcb -> priority) {
        return true;
    } else {
        if(cur -> pcb -> jobLengthScore > next -> pcb -> jobLengthScore) {
            return true;
        } else {
            return false;
        }
    }
}

void sortReadyQueue() {
    if (head == NULL) return;
    //bubble sort
    QueueNode *cur = head;
    bool sorted = false;
    while (!sorted) {
        sorted = true;
        while (cur -> next != NULL) {
            if (swapNeeded(cur)) {
                sorted = false;
                readyQueueSwapWithNext(cur);
            } else {
                cur = cur -> next;
            }
        }
    }
}

QueueNode *readyQueuePopShortestJob() {
    sortReadyQueue();
    QueueNode *node = readyQueuePopHead();
    return node;
}

int readyQueueGetShortestJobScore() {
    QueueNode *cur  = head;
    int shortest = MAX_INT;
    while (cur != NULL) {
        if (cur -> pcb -> jobLengthScore < shortest) {
            shortest = cur -> pcb -> jobLengthScore;
        }
        cur = cur -> next;
    }
    return shortest;
}

void readyQueuePromote(int score) {
    if (head -> pcb -> jobLengthScore == score) return;

    QueueNode *cur = head;
    QueueNode *next;

    while (cur -> next != NULL) {
        if (cur -> next -> pcb -> jobLengthScore == score) break;
        cur = cur -> next;
    }
    if (cur -> next == NULL || cur -> next -> pcb -> jobLengthScore != score) return;
    next = cur -> next;
    cur -> next = cur -> next -> next;
    next -> next = head;
    head = next;
}