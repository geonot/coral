#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// A simple list implementation
typedef struct {
    int size;
    int capacity;
    void** data;
} List;

void* list_new() {
    List* list = (List*)malloc(sizeof(List));
    list->size = 0;
    list->capacity = 4;
    list->data = (void**)malloc(sizeof(void*) * list->capacity);
    return list;
}

void list_append(void* list_ptr, void* item) {
    List* list = (List*)list_ptr;
    if (list->size == list->capacity) {
        list->capacity *= 2;
        list->data = (void**)realloc(list->data, sizeof(void*) * list->capacity);
    }
    list->data[list->size++] = item;
}

// A simple string implementation
typedef struct {
    int len;
    char* chars;
} String;

void* string_new() {
    String* str = (String*)malloc(sizeof(String));
    str->len = 0;
    str->chars = (char*)malloc(1);
    str->chars[0] = '\0';
    return str;
}

void* string_concat(void* s1_ptr, void* s2_ptr) {
    String* s1 = (String*)s1_ptr;
    String* s2 = (String*)s2_ptr;
    String* new_str = (String*)malloc(sizeof(String));
    new_str->len = s1->len + s2->len;
    new_str->chars = (char*)malloc(new_str->len + 1);
    strcpy(new_str->chars, s1->chars);
    strcat(new_str->chars, s2->chars);
    return new_str;
}

void* string_from_int(long long val) {
    String* str = (String*)malloc(sizeof(String));
    char buf[21];
    sprintf(buf, "%lld", val);
    str->len = strlen(buf);
    str->chars = (char*)malloc(str->len + 1);
    strcpy(str->chars, buf);
    return str;
}

void* string_from_float(double val) {
    String* str = (String*)malloc(sizeof(String));
    char buf[32];
    sprintf(buf, "%f", val);
    str->len = strlen(buf);
    str->chars = (char*)malloc(str->len + 1);
    strcpy(str->chars, buf);
    return str;
}

void print_string(String* s) {
    printf("%s\n", s->chars);
}

// Iterator implementation
typedef struct {
    void* collection;
    int index;
} Iterator;

void* iterator_new(void* collection) {
    Iterator* iter = (Iterator*)malloc(sizeof(Iterator));
    iter->collection = collection;
    iter->index = 0;
    return iter;
}

int iterator_next(void* iter_ptr) {
    Iterator* iter = (Iterator*)iter_ptr;
    List* list = (List*)iter->collection;
    return iter->index < list->size;
}

void* iterator_get_value(void* iter_ptr) {
    Iterator* iter = (Iterator*)iter_ptr;
    List* list = (List*)iter->collection;
    return list->data[iter->index++];
}

// Store implementation
void store_save(void* key, void* value) {
    // For now, this is a no-op
}

void* store_load(void* key) {
    // For now, this is a no-op
    return NULL;
}