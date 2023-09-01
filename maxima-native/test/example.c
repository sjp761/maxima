#include <stdio.h>
#include <unistd.h>
#include <stdint.h>
#include <Windows.h>

// Concurrency Functions
typedef size_t (*maxima_create_runtime_t)(void** runtime_out);

// Service Functions
typedef size_t (*maxima_is_service_valid_t)(uint8_t* valid_out);
typedef size_t (*maxima_is_service_running_t)(uint8_t* running_out);
typedef size_t (*maxima_register_service_t)();
typedef size_t (*maxima_start_service_t)(void** runtime);
typedef uint8_t (*maxima_check_registry_validity_t)();
typedef size_t (*maxima_request_registry_setup_t)(void** runtime);

// Authentication Functions
typedef size_t (*maxima_login_t)(void** runtime, const char** token_out);

// Maxima-Object Functions
typedef void* (*maxima_mx_create_t)();
typedef size_t (*maxima_mx_set_access_token_t)(void** runtime, void** mx, const char* token);
typedef size_t (*maxima_mx_start_lsx_t)(void** runtime, void** mx);
typedef size_t (*maxima_mx_consume_lsx_events_t)(void** runtime, void** mx, char*** events_out, unsigned int* event_count_out);
typedef size_t (*maxima_mx_free_lsx_events_t)(char** events, unsigned int event_count);

// Game Functions
typedef size_t (*maxima_launch_game_t)(void** runtime, void** mx, const char* offer_id);

#define DefineProc(mod, type) type##_t type = (type##_t) GetProcAddress(mod, #type);
#define ValidateRet(func) { size_t code = func; if (code != 0) { printf("Failure: %s/%d\n", #func, code); return 0; } }

int main() {
	HMODULE mod = LoadLibrary("maxima.dll");

    // Concurrency Functions
    DefineProc(mod, maxima_create_runtime);

    // Service Functions
    DefineProc(mod, maxima_is_service_valid);
    DefineProc(mod, maxima_is_service_running);
    DefineProc(mod, maxima_register_service);
    DefineProc(mod, maxima_start_service);
    DefineProc(mod, maxima_check_registry_validity);
    DefineProc(mod, maxima_request_registry_setup);

    // Maxima Object Functions
    DefineProc(mod, maxima_mx_create);
    DefineProc(mod, maxima_mx_set_access_token);
    DefineProc(mod, maxima_mx_start_lsx);
    DefineProc(mod, maxima_mx_consume_lsx_events);
    DefineProc(mod, maxima_mx_free_lsx_events);

    // Authentication Functions
    DefineProc(mod, maxima_login);

    // Game Functions
    DefineProc(mod, maxima_launch_game);

    void* runtime;
    ValidateRet(maxima_create_runtime(&runtime));

    printf("Validating service...\n");

    uint8_t serviceValid;
    ValidateRet(maxima_is_service_valid(&serviceValid));

    if (!serviceValid) {
        printf("Registering service...\n");
        ValidateRet(maxima_register_service());
        sleep(1);
    }

    printf("Ensuring service is running...\n");

    uint8_t serviceRunning;
    ValidateRet(maxima_is_service_running(&serviceRunning));

    if (!serviceRunning) {
        printf("Starting service...\n");
        ValidateRet(maxima_start_service(&runtime));
    }

    if (!maxima_check_registry_validity())
    {
        printf("Requesting registry setup\n");
        ValidateRet(maxima_request_registry_setup(&runtime));
    }

    const char* token = NULL;
    ValidateRet(maxima_login(&runtime, &token));

    void* maxima = maxima_mx_create();
    ValidateRet(maxima_mx_set_access_token(&runtime, &maxima, token));
    printf("Starting LSX server...\n");

    ValidateRet(maxima_mx_start_lsx(&runtime, &maxima));

    printf("Launching game...\n");
    ValidateRet(maxima_launch_game(&runtime, &maxima, "Origin.OFR.50.0001523"));

    while (1) {
        char** events;
        unsigned int event_count;
        ValidateRet(maxima_mx_consume_lsx_events(&runtime, &maxima, &events, &event_count));

        for (int i = 0; i < event_count; i++)
        {
            const char* event = events[i];
            printf("LSX Event: %s\n", event);
        }

        maxima_mx_free_lsx_events(events, event_count);
        sleep(0.05);
    }

    printf("Done");
    return 0;
}