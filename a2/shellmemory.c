#include<stdlib.h>
#include<string.h>
#include<stdio.h>
#include <limits.h>


#define FRAME_PAGE_SIZE 3//Each frame is defined as 3 lines long
#define FRAME_STORE_SIZE framesize //GET THIS FROM COMPILER FLAGS
#define VARIABLE_STORE_SIZE varmemsize //GET THIS FROM COMPILER FLAGS

int accessBit = 0;

struct frameStruct {
    char* frameLines[FRAME_PAGE_SIZE];
    int accessBit;
};

int updateAccessBit() {
    return accessBit++;
}

struct variableStruct { //Ripped from spec
    char* varName;
    char* varValue;
};

struct shellMemoryStruct{ //Shell memory consists of two arrays, frame and variables
	struct frameStruct frameStore[FRAME_STORE_SIZE / FRAME_PAGE_SIZE];
    struct variableStruct variableStore[VARIABLE_STORE_SIZE];
};

struct shellMemoryStruct shellMemory;

int findNextEmptyFrame();
void clearFrame(int frameIndex);

int match(char *model, char *var) { //SPEC PROVIDED
	int i, len = strlen(var), matchCount=0;
	for(i = 0; i < len; i++)
		if (*(model+i) == *(var+i)) matchCount++;
	if (matchCount == len) {
        return 1;
    } else {
        return 0;
    }
}

char *extract(char *model) { //SPEC PROVIDED
	char token='=';    // look for this to find value
	char value[1000];  // stores the extract value
	int i,j, len=strlen(model);
	for(i=0;i<len && *(model+i)!=token;i++); // loop till we get there
	// extract the value
	for(i=i+1,j=0;i<len;i++,j++) value[j]=*(model+i);
	value[j]='\0';
	return strdup(value);
}


// Shell memory functions
//Initializes the shellMemory to all nulls
void memInit() {
    //Init all frames to null
    for (int i = 0; i < FRAME_STORE_SIZE / FRAME_PAGE_SIZE; i++) {
        for(int j = 0; j < FRAME_PAGE_SIZE; j++) {
            shellMemory.frameStore[i].frameLines[j] = NULL;
        }
    }
    //Init all variables to "none", "none"
	for (int i = 0; i < VARIABLE_STORE_SIZE; i++) {
		shellMemory.variableStore[i].varName = "none";
		shellMemory.variableStore[i].varValue = "none";
	}
}

//Clears the entirety of the variable store (#3)
void resetVarMem() {
    for (int i = 0; i < VARIABLE_STORE_SIZE; i++) {
        shellMemory.variableStore[i].varName = "none";
        shellMemory.variableStore[i].varValue = "none";
    }
}

// Set key value pair (FOR VARIABLE STORE ONLY)
void memSetValue(char *var_in, char *value_in) {
	for (int i = 0; i < VARIABLE_STORE_SIZE; i++) {
		if (strcmp(shellMemory.variableStore[i].varName, var_in) == 0) { //We found a match for the var in the existing memory
			shellMemory.variableStore[i].varValue = strdup(value_in);
			return;
		} 
	}

	for (int i = 0; i < VARIABLE_STORE_SIZE; i++) { //Var does not exist, need to find a free spot.
		if (strcmp(shellMemory.variableStore[i].varName, "none") == 0){
            shellMemory.variableStore[i].varName = strdup(var_in);
            shellMemory.variableStore[i].varValue = strdup(value_in);
			return;
		} 
	}
	return;
}

//Get value based on input key (FOR VARIABLE STORE ONLY)
char *memGetValue(char *var_in) {
	for (int i = 0; i < VARIABLE_STORE_SIZE; i++) {
		if (strcmp(shellMemory.variableStore[i].varName, var_in) == 0){ //Var found, return its value
			return strdup(shellMemory.variableStore[i].varValue);
		} 
	}
	return NULL; //Var not found
}

//Debugging tool that prints all contents of the shellMemory
void printShellMemory() {
    int emptyFrameCount = 0;
    //Print contents of frameStore
    for (int i = 0; i < FRAME_STORE_SIZE / FRAME_PAGE_SIZE; i++) {
        printf("Frame Entry: %d, Access Bit: %d\n", i, shellMemory.frameStore[i].accessBit);
        int emptyFrame = 0;
        for (int j = 0; j < FRAME_PAGE_SIZE; j++) {
            if (shellMemory.frameStore[i].frameLines[j] != NULL) {
                emptyFrame = 1;
                printf("Contents of line %d: %s\n", j, shellMemory.frameStore[i].frameLines[j]);
            } else {
                printf("Contents of line %d: NULL\n", j);
            }
        }
        if (emptyFrame == 0) {
            emptyFrameCount++;
        }
    }
    printf("Frame store size: %d, Frames in use: %d, Frames free: %d\n", FRAME_STORE_SIZE / FRAME_PAGE_SIZE, (FRAME_STORE_SIZE / FRAME_PAGE_SIZE )-emptyFrameCount, emptyFrameCount);

    //Print contents of varStore
    int emptyVarCount = 0;
    for (int i = 0; i < VARIABLE_STORE_SIZE; i++) {
        if (strcmp(shellMemory.variableStore[i].varName, "none") == 0) {
            emptyVarCount++;
        } else {
            printf("Entry %d: varName: %s\t\tvarValue: %s\n", i, shellMemory.variableStore[i].varName, shellMemory.variableStore[i].varValue);
        }
    }
    printf("Var store size: %d, Entries in use: %d, Entries free: %d\n", VARIABLE_STORE_SIZE, VARIABLE_STORE_SIZE-emptyVarCount, emptyVarCount);
}

//Load a script into the frameStore given a file pointer and size of file in lines
//Returns an array of the indices of the frames that are allocated to the script
int* loadFile(FILE* fp, int numOfLines) {
    //Calculate number of frames memory to be allocated, round up
    int allocatedFramesForMalloc = (numOfLines + (FRAME_PAGE_SIZE - 1)) / FRAME_PAGE_SIZE; //Stolen from StackOverflow for rounding up division
    int* allocatedFrameIndices = malloc(allocatedFramesForMalloc * sizeof(int)); //Array to store the allocated indices

    int allocatedFrames = (numOfLines + (FRAME_PAGE_SIZE - 1)) / FRAME_PAGE_SIZE;
    if (allocatedFrames > 2) { // Limit the maximum frame allocation to 2
        allocatedFrames = 2;
    }

    //For each allocated frame, find a free frame
    for(int i = 0; i < allocatedFrames; i++) {
        int frameIndex = findNextEmptyFrame();

        if (frameIndex == -1) { //No frame found case
            // No need to do anything here, the loadPage function will handle eviction
        } else { //Empty frame found
            for (int j = 0; j < FRAME_PAGE_SIZE; j++) { // Load to frame store from the fp file
                char line[100];
                if (fgets(line, sizeof(line), fp) != NULL) { //We have not yet reached the end of the file
                    shellMemory.frameStore[frameIndex].accessBit = updateAccessBit();
                    shellMemory.frameStore[frameIndex].frameLines[j] = malloc(sizeof(char) * (strlen(line) + 1));
                    strcpy(shellMemory.frameStore[frameIndex].frameLines[j], line);
                }
            }
        }
        allocatedFrameIndices[i] = frameIndex; //Write the allocated frame index
    }

    // Set the remaining elements of allocatedFrameIndices to -1 to which represents that they are not allocated
    for (int i = allocatedFrames; i < allocatedFramesForMalloc; i++) {
        allocatedFrameIndices[i] = -1;
    }

    // Return the array of allocated frame indices
    return allocatedFrameIndices;
}

// Load a single page into the frameStore given a file pointer and the page number
int loadPage(FILE* fp, int pageNum) {
    int* allocatedFrameIndices = malloc(sizeof(int) * 1);
    int frameIndex = findNextEmptyFrame();

    // If no empty frames are found
    if (frameIndex == -1) {

        // Find the frame with the lowest access bit value (least recently used)
        int minAccessBit = INT_MAX; // Use max integer so that any comparison will be less
        int evictFrameIndex = -1;

        // Find the frame with the lowest access bit value, signifying the least recently used frame
        for (int i = 0; i < FRAME_STORE_SIZE / FRAME_PAGE_SIZE; i++) {
            if (shellMemory.frameStore[i].accessBit < minAccessBit) {
                minAccessBit = shellMemory.frameStore[i].accessBit;
                evictFrameIndex = i;
            }
        }

        // Evict the frame with the lowest access bit value by setting its lines to NULL
        printf("Page fault! Victim page contents:\n");
        for (int j = 0; j < FRAME_PAGE_SIZE; j++) {
            if (shellMemory.frameStore[evictFrameIndex].frameLines[j] != NULL) {
                printf("%s", shellMemory.frameStore[evictFrameIndex].frameLines[j]);
                free(shellMemory.frameStore[evictFrameIndex].frameLines[j]);
                shellMemory.frameStore[evictFrameIndex].frameLines[j] = NULL;
            }
        }
        printf("End of victim page contents.\n");

        // Allocate the newly evicted frame
        frameIndex = evictFrameIndex;
    }


    // Seek to the page line associated with the pageNum in the file, representing the backingStore
    int linesToSkip = (pageNum - 1) * FRAME_PAGE_SIZE;
    for (int i = 0; i < linesToSkip; i++) {
        char line[100];
        if (fgets(line, sizeof(line), fp) == NULL) {
            // Error
        }
    }

    // Load one page of lines starting from the backing store by writing it to the frameStore
    for (int j = 0; j < FRAME_PAGE_SIZE; j++) {
        char line[100];
        if (fgets(line, sizeof(line), fp) != NULL) {
            shellMemory.frameStore[frameIndex].accessBit = updateAccessBit();
            shellMemory.frameStore[frameIndex].frameLines[j] = malloc(sizeof(char) * (strlen(line) + 1));
            strcpy(shellMemory.frameStore[frameIndex].frameLines[j], line);
        }
    }

    // Return the frame index of the newly allocated frame
    return frameIndex;
}


//Returns the line @ the address
//Despite the 2D nature of the frameStore, we only need the "absolute" value of the address we are trying to reach
//2 -> will access frameStore[0].frameLines[2]
//4 -> will access frameStore[1].frameLines[1]
char* frameGetValueAtLine(int address) {
    int frame = address / FRAME_PAGE_SIZE;
    int withinFrame = address % FRAME_PAGE_SIZE;
    
    // Update the access bit for LRU
    shellMemory.frameStore[frame].accessBit = updateAccessBit();
    // Returns the command line
    return shellMemory.frameStore[frame].frameLines[withinFrame];
}

//Clears a single frame
void clearFrame(int frameIndex) {
    for (int i = 0; i < FRAME_PAGE_SIZE; i++) {
        shellMemory.frameStore[frameIndex].accessBit = updateAccessBit();
        shellMemory.frameStore[frameIndex].frameLines[i] = NULL;
    }
}

//Given a list of indices, clears the frames at those indices
void clearSetOfFrames(int frames[], int numOfFrames){
    for(int i = 0; i < numOfFrames; i++) {
        clearFrame(frames[i]);
    }
}

//Check if a frame is empty (ALL LINES NULL) given an index
//Returns 1 if empty, 0 if not
int is_frame_empty(int index) {
    for(int i = 0; i < FRAME_PAGE_SIZE; i++){
        if (shellMemory.frameStore[index].frameLines[i] != NULL) { //Line not empty
            return 0;
        }
    }
    return 1;
}

//Finds the next empty frame (FRONT TO BACK)
//Returns the index of the found frame if it exists, -1 if not
int findNextEmptyFrame(){
    for(int i = 0; i < FRAME_STORE_SIZE / FRAME_PAGE_SIZE; i++) {
        if (is_frame_empty(i) == 1) { //We found an empty frame
            return i;
        }
    }
    return -1; //We found nothing :(
}