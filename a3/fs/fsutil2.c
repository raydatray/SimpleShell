#include "fsutil2.h"
#include "bitmap.h"
#include "cache.h"
#include "debug.h"
#include "directory.h"
#include "file.h"
#include "filesys.h"
#include "free-map.h"
#include "fsutil.h"
#include "inode.h"
#include "off_t.h"
#include "partition.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define BUFFER_SIZE 1024 //TO BE CHANGED ?

int copy_in(char *fname) {
  // Open the file
  FILE* sourceFilePointer = fopen(fname, "r");

  //Check the validity of the returned pointer
  if (sourceFilePointer == NULL) {
    printf("Error: file not found\n");
    return -1;
    // TODO: return the correct error code from interpreter.c
  }

  //Trim the filename to retain only the name of the file
  char* targetFileName = strrchr(fname, '/');
  if (targetFileName == NULL) {
    targetFileName = fname;
  } else {
    targetFileName++;
  }

  printf("Target file name: %s\n", targetFileName);

  //Get full size of source file
  int sourceFileSize;

  int currentPos = ftell(sourceFilePointer);
  fseek(sourceFilePointer, 0, SEEK_END);
  sourceFileSize = ftell(sourceFilePointer);
  fseek(sourceFilePointer, currentPos, SEEK_SET);

  printf("Source file size: %d\n", sourceFileSize);

  //Calcualte the maximum available freespace in the FS



  //Open the file in the FS
  struct file* targetFilePointer = filesys_open(targetFileName); //Error handling?

  //Start writing, read a single char and write it to the target file
  char buffer[BUFFER_SIZE];
  int bytesWritten = 0;

  while (fgets(buffer, BUFFER_SIZE, sourceFilePointer) != NULL) {
    printf("Buffer: %s, Length of buffer: %llu\n", buffer, strlen(buffer));
    int actualBytesWritten = file_write(targetFilePointer, buffer, strlen(buffer) + 1); //+1 to include \0 (added in file_write)
    //TODO: DO WE INCLUDE THE \0 IN THE BYTE COUNT?
    if (actualBytesWritten < strlen(buffer) + 1) { //To account for the size of the \0
        bytesWritten += actualBytesWritten;
        printf("Warning: could only write %d out of %d bytes (reached end of file)\n", bytesWritten, sourceFileSize);
        return -1; //TODO: A REAL ERROR CODE
    }
    bytesWritten += actualBytesWritten;
  }
  printf("Bytes written: %d\n", bytesWritten);
  return 0;
}

int copy_out(char *fname) {
  //Open the file from the FS
  struct file* sourceFilePointer = filesys_open(fname);

  //Open the file to write to
  FILE* targetFilePointer = fopen(fname, "w");

  //Check the validity of the returned pointer
  if (sourceFilePointer == NULL) return -1;

  char buffer[BUFFER_SIZE];
  int offset = 0;
  int bytesRead;

  while((bytesRead = file_read_at(sourceFilePointer, buffer,BUFFER_SIZE, offset)) > 0) {
    printf("Read from file: %s", buffer);
    fwrite(buffer, sizeof(char), strlen(buffer), targetFilePointer);
    offset += bytesRead;
  }

  return 0;
}

void find_file(char *pattern) {
  //Iterate over all files in the directory
  //For each file, check if the pattern is contained within it

  struct dir *dir = dir_open_root();
  char name[NAME_MAX + 1]; //MAX FILE NAME LENGTH?

  while(dir_readdir(dir, name)) {
      char buffer[BUFFER_SIZE];
      int offset = 0;
      int bytesRead;

      struct file *file = get_file_by_fname(name);

      if (file == NULL) {
        file = filesys_open(name);
      }

        while((bytesRead = file_read_at(file, buffer,BUFFER_SIZE, offset)) > 0) {
            if (strstr(buffer, pattern) != NULL) {
                printf("%s\n", name);
                break;
            }
            offset += bytesRead;
        }

  }
  dir_close(dir);
}

void fragmentation_degree() {
  //Iterate over all files in the directory
  //For each file, access its inodes and check which sector it is located in
  //Store the previous sector, and check if it is more than 3 away from the next one

  struct dir *dir = dir_open_root();
  char name[NAME_MAX + 1]; //MAX FILE NAME LENGTH?

  int fragmentedFiles = 0;
  int totalFiles = 0;

  while(dir_readdir(dir, name)) {
    struct file* file = get_file_by_fname(name); //Attempt to load from memory

    if (file == NULL) {
      file = filesys_open(name); //Attempt to load from disk
    }

    add_to_file_table(file, name);

    struct inode* inode = file_get_inode(file);

    //Iterate over direct blocks
    struct inode_disk inodeData = inode -> data;
    int previousSectorNumber = inodeData.direct_blocks[0];


    //DIRECT BLOCKS
    for(int i = 1; i < DIRECT_BLOCKS_COUNT; i++) {
      if (inodeData.direct_blocks[i] == 0) { //We've reached an unallocated sector
        goto endWhileLoop; //Immediately force next iteration
      } else {
        if (inodeData.direct_blocks[i] - previousSectorNumber > 3) {
          fragmentedFiles++;
          goto endWhileLoop; //Immediately force next iteration
        }
        previousSectorNumber = inodeData.direct_blocks[i];
      }
    }


    //INDIRECT BLOCKS DEGREE 1 (A BLOCK FULL OF POINTERS)
    if (inodeData.indirect_block != 0) { //If indirect block is allocated
      block_sector_t indirectBlockPointers[INDIRECT_BLOCKS_PER_SECTOR]; //Array of pointers to blocks
      buffer_cache_read(inodeData.indirect_block, &indirectBlockPointers); //Read the indirect block sector into the pointer

      for(int i = 0; i < INDIRECT_BLOCKS_PER_SECTOR; i++) {
        if (indirectBlockPointers[i] == 0) { //We've reached an unallocated sector
          goto endWhileLoop; //Immediately force next iteration
        } else {
          if (indirectBlockPointers[i] - previousSectorNumber > 3) {
            fragmentedFiles++;
            goto endWhileLoop; //Immediately force next iteration
          }
          previousSectorNumber = indirectBlockPointers[i];
        }
      }
    }

    //INDIRECT BLOCKS DEGREE 2 (A BLOCK FULL OF POINTERS TO BLOCKS FULL OF POINTERS)
    if (inodeData.doubly_indirect_block != 0) { //If doubly indirect block is allocated
      block_sector_t doublyIndirectBlockPointers[INDIRECT_BLOCKS_PER_SECTOR]; //Array of pointers to blocks
      buffer_cache_read(inodeData.doubly_indirect_block, &doublyIndirectBlockPointers); //Read the doubly indirect block sector into the pointer

      for(int i = 0 ; i < INDIRECT_BLOCKS_PER_SECTOR; i++) {
        if (doublyIndirectBlockPointers[i] == 0) { //We've reached an unallocated sector
          goto endWhileLoop; //Immediately force next iteration
        } else {
          block_sector_t indirectBlockPointers[INDIRECT_BLOCKS_PER_SECTOR]; //Array of pointers to blocks
          buffer_cache_read(doublyIndirectBlockPointers[i], &indirectBlockPointers); //Read the indirect block sector into the pointer

          for(int j = 0; j < INDIRECT_BLOCKS_PER_SECTOR; j++) {
            if (indirectBlockPointers[j] == 0) { //We've reached an unallocated sector
              goto endWhileLoop; //Immediately force next iteration
            } else {
              if (indirectBlockPointers[j] - previousSectorNumber > 3) {
                fragmentedFiles++;
                goto endWhileLoop; //Immediately force next iteration
              }
              previousSectorNumber = indirectBlockPointers[j];
            }
          }
        }
      }
    }
    endWhileLoop : ;
    totalFiles++;
  }
  printf("Num fragmentable files: %d\n", totalFiles);
  printf("Num fragmented files: %d\n", fragmentedFiles);
  printf("Fragmentation pct: %f\n", (float)fragmentedFiles/totalFiles);
}

int defragment() {
  struct tempFile {
    char fileName[NAME_MAX + 1];
    char* content;
  };

  int fileCount = 0;
  char fileName[NAME_MAX + 1];

  //Count out how many files we need to malloc for
  struct dir *dir = dir_open_root();
  while(dir_readdir(dir, fileName)) {
    fileCount++;
  }
  dir_close(dir);

  struct tempFile* files = malloc(fileCount * sizeof(struct tempFile));

  //Copy each file into memory and then delete it from the FS
  int i = 0;
  dir = dir_open_root();
  while(dir_readdir(dir, fileName)) {
    struct file* file = filesys_open(fileName); //Read straight from disk

    if (file != NULL) {
      files[i].content = malloc(file_length(file) * sizeof(char));
      file_read(file, files[i].content, file_length(file));
      strcpy(files[i].fileName, fileName);
      i++;
    }
    filesys_remove(fileName);
  }
  dir_close(dir);

  //We should have an empty disk now (NOT FORMATTED HOWEVER)
  //This allows for recovery if we want
  fsutil_freespace();

  for(int j = 0; j < fileCount; j++) {
    printf("%s\n", files[j].fileName);
  }

  //Recreate the files
  for (int j = 0; j < fileCount; j++) {
    filesys_create(files[j].fileName, strlen(files[j].content), false);
    struct file* file = filesys_open(files[j].fileName);
    file_write(file, files[j].content, strlen(files[j].content));
  }

  return 0;
}

void recover(int flag) {
  if (flag == 0) { // recover deleted inodes
    for(size_t i = 0; i < bitmap_size(free_map); i++) {
      //printf("Bit %zu: %d\n", i, bitmap_test(free_map, i));
      if (bitmap_test(free_map, i) == 0) {
        //printf("Sector %zu is free\n", i);

        struct inode_disk recoveredInode;
        buffer_cache_read(i, &recoveredInode);

        if (recoveredInode.magic == INODE_MAGIC) {
          //printf("Inode %zu is recoverable\n", i);

          char recoveredFileName[NAME_MAX + 1];
          sprintf(recoveredFileName, "recovered0-%zu", i);

          filesys_create(recoveredFileName, recoveredInode.length, recoveredInode.is_dir);

          struct file* recoveredFile = filesys_open(recoveredFileName);
          block_sector_t* dataSectors = get_inode_data_sectors(recoveredFile -> inode);
          size_t numSectors = bytes_to_sectors(recoveredInode.length);

          for(size_t j = 0; j < numSectors; j++) {
            char buffer[BLOCK_SECTOR_SIZE];
            buffer_cache_read(dataSectors[j], buffer);
            file_write(recoveredFile, buffer, strlen(buffer));
          }

          file_close(recoveredFile);

        }
      }
    }
  } else if (flag == 1) { // recover all non-empty sectors


    // TODO
  } else if (flag == 2) { // data past end of file.

    // TODO
  }
}