#include <stdio.h>
#include <stdlib.h>
#include <string.h> 
#include <unistd.h>
#include <sys/stat.h>
#include <stdbool.h>

#include "shellmemory.h"
#include "shell.h"
#include "kernel.h"
#include "ready_queue.h"
#include "interpreter.h"

int MAX_ARGS_SIZE = 7;

char* errorMsgs[] = {
	"no error",
	"file does not exist",
	"file could not be loaded",
	"no space left in shell memory",
	"ready queue is full",
	"scheduling policy error",
	"too many tokens",
	"too few tokens",
	"non-alphanumeric token",
	"unknown name",
	"cd",
	"mkdir"
};

int handleError(enum Error errorCode){
	printf("Bad command: %s\n", errorMsgs[errorCode]);
	return errorCode;
}

int help();
int quit();
int set(char* var, char* value);
int print(char* var);
int run(char* script);
int echo(char* var);
int myLs();
int myMkdir(char* dirname);
int myTouch(char* filename);
int myCd(char* dirname);
int exec(char *fname1, char *fname2, char *fname3); //, char* policy, bool background, bool mt);
int resetMem();

// Interpret commands and their arguments
int interpreter(char* commandArgs[], int argSize){
	if (argSize < 1) return handleError(TOO_FEW_TOKENS);
	if (argSize > MAX_ARGS_SIZE) return handleError(TOO_MANY_TOKENS);


	for (int i = 0; i < argSize; i++) { // strip spaces, newline, etc.
		commandArgs[i][strcspn(commandArgs[i], "\r\n")] = 0;
	}

	if (strcmp(commandArgs[0], "help") == 0) { // help
	    if (argSize > 1) return handleError(TOO_MANY_TOKENS);
	    return help();

	} else if (strcmp(commandArgs[0], "quit") == 0 || strcmp(commandArgs[0], "exit") == 0) { // quit
		if (argSize > 1) return handleError(TOO_MANY_TOKENS);
		return quit();

	} else if (strcmp(commandArgs[0], "set") == 0) { //set
		if (argSize < 3) return handleError(TOO_FEW_TOKENS);


		int totalLen = 0;

		for(int i = 2; i < argSize; i++){
			totalLen += strlen(commandArgs[i]) + 1;
		}

		char *value = (char*) calloc(1, totalLen);
		char spaceChar = ' ';

		for(int i = 2; i < argSize; i++){
			strncat(value, commandArgs[i], strlen(commandArgs[i]));
			if (i < argSize - 1) strncat(value, &spaceChar, 1);

		}

		int errCode = set(commandArgs[1], value);
		free(value);
		return errCode;

	} else if (strcmp(commandArgs[0], "print") == 0) { // print
		if (argSize < 2) return handleError(TOO_FEW_TOKENS);
		if (argSize > 2) return handleError(TOO_MANY_TOKENS);

		return print(commandArgs[1]);

	} else if (strcmp(commandArgs[0], "run") == 0) { // run
		if (argSize < 2) return handleError(TOO_FEW_TOKENS);
		if (argSize > 2) return handleError(TOO_MANY_TOKENS);

		return run(commandArgs[1]);

	} else if (strcmp(commandArgs[0], "echo") == 0) { // echo
		if (argSize > 2) return handleError(TOO_MANY_TOKENS);

		return echo(commandArgs[1]);

	} else if (strcmp(commandArgs[0], "my_ls") == 0) { // ls
		if (argSize > 1) return handleError(TOO_MANY_TOKENS);

		return myLs(commandArgs[0]);

	} else if (strcmp(commandArgs[0], "my_mkdir") == 0) {
		if (argSize > 2) return handleError(TOO_MANY_TOKENS);

		return myMkdir(commandArgs[1]);

	} else if (strcmp(commandArgs[0], "my_touch") == 0) {
		if (argSize > 2) return handleError(TOO_MANY_TOKENS);

		return myTouch(commandArgs[1]);

	} else if (strcmp(commandArgs[0], "my_cd") == 0) {
		if (argSize> 2) return handleError(TOO_MANY_TOKENS);

		return myCd(commandArgs[1]);

	} else if (strcmp(commandArgs[0], "exec") == 0) {
		if (argSize <= 1) return handleError(TOO_FEW_TOKENS);
		if (argSize > 5) return handleError(TOO_MANY_TOKENS);

		if (argSize == 2) {
            return exec(commandArgs[1],NULL,NULL);
        } else if (argSize == 3) {
            return exec(commandArgs[1],commandArgs[2],NULL);
        } else if (argSize == 4) {
            return exec(commandArgs[1],commandArgs[2],commandArgs[3]);
        }
	} else if (strcmp(commandArgs[0], "resetmem")==0) {
		return resetMem();
	}
	return handleError(BAD_COMMAND);
}

int help(){

	char helpString[] = "COMMAND			DESCRIPTION\n \
help			Displays all the commands\n \
quit			Exits / terminates the shell with “Bye!”\n \
set VAR STRING		Assigns a value to shell memory\n \
print VAR		Displays the STRING assigned to VAR\n \
run SCRIPT.TXT		Executes the file SCRIPT.TXT\n ";
	printf("%s\n", helpString);
	return 0;
}

int quit(){
	printf("%s\n", "Bye!");
	readyQueueDestroy();
	system("rm -rf ./backingStore"); // Remove backing store
	exit(0);
}

int set(char* var, char* value){
	char *link = "=";
	char buffer[1000];
	strcpy(buffer, var);
	strcat(buffer, link);
	strcat(buffer, value);
	memSetValue(var, value);
	return 0;
}

int print(char* var){
	char *value = memGetValue(var);
    if(value == NULL) {
        return 0;
    }
	printf("%s\n", value); 
	return 0;
}

int echo(char* var){
	if(var[0] == '$') print(++var);
	else printf("%s\n", var); 
	return 0; 
}

int myLs(){
	int errCode = system("ls | sort");
	return errCode;
}

int myMkdir(char *dirname){
	char *dir = dirname;
	if(dirname[0] == '$'){
		char *value = memGetValue(++dirname);
		if(value == NULL || strchr(value, ' ') != NULL){
			return handleError(ERROR_MKDIR);
		}
		dir = value;
	}
	int nameLen = strlen(dir);
	char* command = (char*) calloc(1, 7 + nameLen);
	strncat(command, "mkdir ", 7);
	strncat(command, dir, nameLen);
	int errCode = system(command);
	free(command);
	return errCode;
}

int myTouch(char* filename) {
	int nameLen = strlen(filename);
	char* command = (char*) calloc(1, 7 + nameLen);
	strncat(command, "touch ", 7);
	strncat(command, filename, nameLen);
	int errCode = system(command);
	free(command);
	return errCode;
}

int myCd(char* dirname) {
	struct stat info;
	if(stat(dirname, &info) == 0 && S_ISDIR(info.st_mode)) {
		//the object with dirname must exist and is a directory
		int errCode = chdir(dirname);
		return errCode;
	}
	return handleError(ERROR_CD);
}

int run(char* script) {
	int errCode = 0;

	errCode = processInitialize(script);
	if (errCode == 11) {
		return handleError(errCode);
	}

	scheduleByPolicy("FCFS");
	return errCode;
}

int exec(char *fname1, char *fname2, char *fname3) {
	int error_code = 0;

	if(fname1 != NULL){
        error_code = processInitialize(fname1);
		if(error_code != 0){
			return handleError(error_code);
		}
    }
    if(fname2 != NULL){
        error_code = processInitialize(fname2);
		if(error_code != 0){
			return handleError(error_code);
		}
    }
    if(fname3 != NULL){
        error_code = processInitialize(fname3);
		if(error_code != 0){
			return handleError(error_code);
		}
    } 
	error_code = scheduleByPolicy("RR");
	if(error_code != 0){
		return handleError(error_code);
	}
}

int resetMem() {
	resetVarMem();
	return 0;
}
