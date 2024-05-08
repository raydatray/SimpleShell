#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h> // for my_cd
#include <sys/stat.h> // for my_mkdir
#include "shellmemory.h"
#include "shell.h"

int MAX_ARGS_SIZE = 7; //Change to 7 for enchanced set

int badcommand(){
	printf("%s\n", "Unknown Command");
	return 1;
}

int badcommandSet(){
	printf("%s\n", "Bad command: set");
	return 1;
}

int badcommandCd(){
	printf("%s\n", "Bad command: my_cd");
	return 1;
}

// For run command only
int badcommandFileDoesNotExist(){
	printf("%s\n", "Bad command: File not found");
	return 3;
}

int help();
int quit();
int set(char* var, char* value);
int print(char* var);
int run(char* script);
int touch(char* filename);
int cat(char* filename);
int badcommandFileDoesNotExist();
int echo(char* argument);
int my_mkdir(char* directory);
int my_cd(char* directory);


// Interpret commands and their arguments
int interpreter(char* command_args[], int args_size){
	int i;

	if ( args_size < 1 ){
		return badcommand();
	}

	for ( i=0; i<args_size; i++){ //strip spaces new line etc
		command_args[i][strcspn(command_args[i], "\r\n")] = 0;
	}

	if (strcmp(command_args[0], "help")==0){
	    //help
	    if (args_size != 1) return badcommand();
	    return help();
	
	} else if (strcmp(command_args[0], "quit")==0) {
		//quit
		if (args_size != 1) return badcommand();
		return quit();

	} else if (strcmp(command_args[0], "set")==0) {
		//set
		if ((args_size < 3) || (args_size > 7)) return badcommandSet(); //change to 1 <= x <= 5

		int totalLength = 0;

		for(int i = 2; i < args_size; i++){
			totalLength += strlen(command_args[i]) + 1;
		}

		char* allocatedString = malloc(totalLength * sizeof(char)); //allocate the string formed from tokens
		
		allocatedString[0] = '\0';
		
		for(int i = 2; i < args_size; i ++){
			strcat(allocatedString, command_args[i]);
			if (i < args_size - 1) {
				strcat(allocatedString, " ");
			}
		}

		return set(command_args[1], allocatedString);
	
	} 
	else if (strcmp(command_args[0], "echo")==0) {
		// echo
		if (args_size != 2) return badcommand();
		return echo(command_args[1]);
	}
	else if (strcmp(command_args[0], "my_mkdir")==0) {
		// my_mkdir
		if (args_size != 2) return badcommand();
		return my_mkdir(command_args[1]);
	
	}
	else if (strcmp(command_args[0], "my_cd")==0) {
		// my_cd
		if (args_size != 2) return badcommand();
		return my_cd(command_args[1]);
	
	}
	else if (strcmp(command_args[0], "print")==0) {
		if (args_size != 2) return badcommand();
		return print(command_args[1]);
	
	} else if (strcmp(command_args[0], "run")==0) {
		if (args_size != 2) return badcommand();
		return run(command_args[1]);
	
	} else if (strcmp(command_args[0], "my_ls")==0) {
		if (args_size != 1) return badcommand();
		return system("ls");

	} else if (strcmp(command_args[0], "my_touch")==0) {
		if (args_size != 2) return badcommand();
		return touch(command_args[1]);

	} else if (strcmp(command_args[0], "my_cat")==0) {
		if (args_size != 2) return badcommand();
		return cat(command_args[1]);

	} else {
		return badcommand();
	}
}

int help(){

	char help_string[] = "COMMAND			DESCRIPTION\n \
help			Displays all the commands\n \
quit			Exits / terminates the shell with “Bye!”\n \
set VAR STRING		Assigns a value to shell memory\n \
print VAR		Displays the STRING assigned to VAR\n \
run SCRIPT.TXT		Executes the file SCRIPT.TXT\n ";
	printf("%s\n", help_string);
	return 0;
}

int quit(){
	printf("%s\n", "Bye!");
	exit(0);
}

int set(char* var, char* value){
	char *link = "=";
	char buffer[1000];
	strcpy(buffer, var);
	strcat(buffer, link);
	strcat(buffer, value);

	mem_set_value(var, value);

	return 0;

}

int echo(char* argument) {
	
	if (argument[0] == '$') {
		char* result = mem_get_value(argument + 1);
		if (strcmp(result, "Variable does not exist") == 0) {
			printf("%s\n", "Variable does not exist");
			//return -1; Depending on whats expected
		}
		else {
			printf("%s\n", result);
		}
		free(result);
	}
	else {
		printf("%s\n", argument);
	}

	return 0;
	
}

int my_mkdir(char* directory) {
	// 0777 = 111 111 111 for permissions
	if(mkdir(directory, 0777) == -1) {
		// maybe delegate to error function
		printf("%s\n", "Error ");
		return 1; 
	}
	return 0;
}

int my_cd(char* directory) {
	if (chdir(directory) == -1) {
		return badcommandCd();
	}
	return 0;
}
int print(char* var){
	printf("%s\n", mem_get_value(var)); 
	return 0;
}

int run(char* script){
	int errCode = 0;
	char line[1000];
	FILE *p = fopen(script,"rt");  // the program is in a file

	if(p == NULL){
		return badcommandFileDoesNotExist();
	}

	fgets(line,999,p);
	while(1){
		errCode = parseInput(line);	// which calls interpreter()
		memset(line, 0, sizeof(line));

		if(feof(p)){
			break;
		}
		fgets(line,999,p);
	}

    fclose(p);

	return errCode;
}

int touch(char* filename){
	FILE *newFile = fopen(filename, "w");

	if (newFile == NULL){
		printf("%s\n", "Error creating file");
		return 1;
	}

	fclose(newFile);
	return 0;
}

int cat(char* filename){
	FILE *toRead = fopen(filename, "r");

	if (toRead == NULL){
		printf("%s\n", "File does not exist");
		return 1;
	}

	char* line = NULL;
	size_t len = 0;
	ssize_t read;

	while((read = getline(&line, &len, toRead)) !=  -1){
		printf("%s", line);
	}
	free(line);
	fclose(toRead);
	return 0;
}

