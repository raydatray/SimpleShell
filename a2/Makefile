CC=gcc
CFLAGS=

framesize ?= 21
varmemsize ?= 10

myshell: shell.c interpreter.c shellmemory.c
	$(CC) $(CFLAGS) -Dframesize=$(framesize) -Dvarmemsize=$(varmemsize) -c -g shell.c interpreter.c shellmemory.c kernel.c pcb.c ready_queue.c
	$(CC) $(CFLAGS) -o myshell shell.o interpreter.o shellmemory.o kernel.o pcb.o ready_queue.o

clean: 
	rm myshell; rm *.o
