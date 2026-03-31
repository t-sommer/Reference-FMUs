#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "fmi2schema.h"
#include "fmi3schema.h"

#include <libxml/xmlschemas.h>


static size_t n_messages = 0;
static char** messages = NULL;

static void log_error(void* ctx, const char* msg, ...) {
    
    (void)ctx; // unused

    // Reallocate messages array for new message
    char** temp = (char**)realloc(messages, (n_messages + 1) * sizeof(char*));

    if (!temp) return;

    messages = temp;
    messages[n_messages] = NULL;

    va_list args;

    va_start(args, msg);
    const size_t len = vsnprintf(NULL, 0, msg, args);
    va_end(args);

    char* message = (char*)malloc(len + 1);

    if (!message) return;

    va_start(args, msg);
    vsnprintf(message, len + 1, msg, args);
    va_end(args);

    messages[n_messages] = message;

    n_messages++;
}

static void clear_messages() {

    for (size_t i = 0; i < n_messages; i++) {
        free(messages[i]);
    }

    free(messages);

    n_messages = 0;
    messages = NULL;
}


int validate_model_description(const char* model_description_path, int fmi_major_version, char*** messages) {

    clear_messages();
    
    xmlDocPtr doc = xmlParseFile(model_description_path);

    if (!doc) {
        log_error(NULL, "Invalid XML.");
        goto TERMINATE;
    }

    xmlNodePtr root = xmlDocGetRootElement(doc);

    if (root == NULL) {
        log_error(NULL, "Empty document.");
        goto TERMINATE;
    }

    xmlSchemaParserCtxtPtr pctxt = NULL;

    if (fmi_major_version == 2) {
        pctxt = xmlSchemaNewMemParserCtxt((char*)fmi2Merged_xsd, fmi2Merged_xsd_len);
    }
    else if (fmi_major_version == 3) {
        pctxt = xmlSchemaNewMemParserCtxt((char*)fmi3Merged_xsd, fmi3Merged_xsd_len);
    }
    else {
        log_error(NULL, "Unsupported FMI version.");
        goto TERMINATE;
    }

    xmlSchemaPtr schema = xmlSchemaParse(pctxt);

    if (schema == NULL) {
        log_error(NULL, "Failed to parse XSD schema.");
        goto TERMINATE;
    }

    xmlSchemaValidCtxtPtr vctxt = xmlSchemaNewValidCtxt(schema);

    if (!vctxt) {
        log_error(NULL, "Failed to create validation context.");
        goto TERMINATE;
    }

    xmlSchemaSetValidErrors(vctxt, (xmlSchemaValidityErrorFunc)log_error, NULL, NULL);

    if (xmlSchemaValidateDoc(vctxt, doc)) {
        goto TERMINATE;
    }
    
TERMINATE:
    
    return n_messages;
}