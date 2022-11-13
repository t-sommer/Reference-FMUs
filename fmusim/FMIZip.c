#include <zip.h>
#include <errno.h>
#ifdef _WIN32
#include <Windows.h>
#include <Shlwapi.h>
#include <process.h>
#include <strsafe.h>
#endif
#include <string.h>
#include <stdlib.h>

#include "FMIZip.h"


const char* FMICreateTemporaryDirectory() {

#ifdef _WIN32
    const char* tempfile = _tempnam(NULL, NULL);

    if (!tempfile) {
        return NULL;
    }

    BOOL res = CreateDirectoryA(tempfile, 0);

    if (!PathFileExistsA(tempfile)) {
        free((void*)tempfile);
        return NULL;
    }

    return tempfile;
#endif
    
    return NULL;
}

int FMIPathAppend(char* path, const char* more) {
#ifdef _WIN32
    return PathAppendA(path, "modelDescription.xml");
#endif
    return 0;
}

int FMIExtractArchive(const char* filename, const char* unzipdir) {

    const char* archive;
    struct zip* za;
    struct zip_file* zf;
    struct zip_stat sb;
    char buf[100];
    int err;
    int i, len;
    FILE* fd;
    long long sum;
    char path[2048];

    if ((za = zip_open(filename, 0, &err)) == NULL) {
        zip_error_to_str(buf, sizeof(buf), err, errno);
        printf("Can't open zip archive `%s': %s\n", filename, buf);
        return 1;
    }

    for (i = 0; i < zip_get_num_entries(za, 0); i++) {

        if (zip_stat_index(za, i, 0, &sb) == 0) {

            len = strlen(sb.name);

            strcpy(path, unzipdir);
            
#ifdef _WIN32
            PathAppendA(path, sb.name);
#else
            
#endif
            if (sb.name[len - 1] == '/') {
#ifdef _WIN32
                mkdir(path);
#else
                
#endif
            } else {

                zf = zip_fopen_index(za, i, 0);

                if (!zf) {
                    exit(100);
                }

                fd = fopen(path, "wb");

                if (fd < 0) {
                    exit(101);
                }

                sum = 0;
                while (sum != sb.size) {
                    len = zip_fread(zf, buf, 100);
                    if (len < 0) {
                        exit(102);
                    }
                    fwrite(buf, len, 1, fd);
                    sum += len;
                }
                fclose(fd);
                zip_fclose(zf);
            }
        } else {
            printf("File[%s] Line[%d]\n", __FILE__, __LINE__);
        }
    }

    if (zip_close(za) == -1) {
        printf("Can't close zip archive `%s'\n", filename);
        return 1;
    }

    return 0;
}

int FMIRemoveDirectory(const char* path) {

    char command[4096];

    sprintf(command, "rmdir \"%s\" /s /q", path);

    return system(command);
}
