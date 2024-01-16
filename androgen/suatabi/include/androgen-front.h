/*
    *  androgen-front.h - Frontend for the Androgen boot corpus 
    *
    *  Copyright (c) 2024 Kıvılcım L. Öztürk
    *  Distributed under the terms of the MIT License.
*/
#ifndef ANDROGEN_FRONT_H
#   define ANDROGEN_FRONT_H
    // Includes
    // for dynamic linkage, use dlfcn.h
#   include <dlfcn.h>
    // for low level reading and writing operations in Linux
#   include <unistd.h>
    // for file operations
#   include <fcntl.h>
#   include <sys/stat.h>
    
/// Represents file handles that will be used by dlfcn.h and unistd.h
typedef struct {
    int fd;
    void *handle;
} androgen_corpus_handle_t;

/// Represents a unit file in the corpus
typedef struct {
    char *name;
    androgen_corpus_handle_t owner;
    size_t size;
    void *data;
} androgen_corpus_unit_t;

/// Opens the corpus
androgen_corpus_handle_t androgen_corpus_open(const char *path);

/// Closes the corpus
void androgen_corpus_close(androgen_corpus_handle_t handle);

/// Loads a unit from the corpus
int androgen_corpus_load_unit(androgen_corpus_handle_t handle, 
               const char *name, androgen_corpus_unit_t *unit);

#endif // ANDROGEN_FRONT_H
