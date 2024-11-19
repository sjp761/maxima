#define ERR_SUCCESS 0

#define ERR_UNKNOWN 1

#define ERR_CHECK_LE 2

#define ERR_LOGIN_FAILED 3

#define ERR_INVALID_ARGUMENT 4

#define ERR_NOT_LOGGED_IN 5

/**
 * Get the last error.
 */
const char *maxima_get_last_error(void);

/**
 * Set up Maxima's logging.
 */
uintptr_t maxima_init_logger(void);

/**
 * Create an asynchronous runtime.
 */
uintptr_t maxima_create_runtime(void **runtime_out);

/**
 * Check if the Maxima Background Service is installed and valid.
 */
uintptr_t maxima_is_service_valid(bool *out);

/**
 * Check if the Maxima Background Service is running.
 */
uintptr_t maxima_is_service_running(bool *out);

/**
 * Register the Maxima Background Service. Runs maxima-bootstrap for admin access.
 */
uintptr_t maxima_register_service(void);

/**
 * Start the Maxima Background Service.
 */
uintptr_t maxima_start_service(void **runtime);

/**
 * Stop the Maxima Background Service.
 */
uintptr_t maxima_stop_service(void **runtime);

/**
 * Check if the Windows Registry is properly set up for Maxima.
 */
bool maxima_check_registry_validity(void);

/**
 * Request the Maxima Background Service to set up the Windows Registry.
 */
uintptr_t maxima_request_registry_setup(void **runtime);

/**
 * Log into an EA account and retrieve an access token. Opens the EA website for authentication.
 */
uintptr_t maxima_login(void **runtime, char **token_out);

/**
 * Log into an EA account with a persona (email/username) and password.
 */
uintptr_t maxima_login_manual(void **runtime, void **mx, const char *persona, const char *password);

/**
 * Retrieve the access token for the currently selected account. Can return [ERR_NOT_LOGGED_IN]
 */
uintptr_t maxima_access_token(void **runtime, void **mx, const char **token_out);

/**
 * Retrieve a nucleus auth code with the specified client id. Can return [ERR_NOT_LOGGED_IN]
 */
uintptr_t maxima_auth_exchange(void **runtime,
                               void **mx,
                               const char *client_id,
                               const char **code_out);

/**
 * Create a Maxima object.
 */
const void *maxima_mx_create(void **runtime);

/**
 * Set the stored token retrieved from [maxima_login].
 */
uintptr_t maxima_mx_set_access_token(void **runtime, const void **mx, const char *token);

/**
 * Set the port to be used for the LSX server. This will be automatically passed to games.
 * Note that not every game supports a custom LSX port, the default is 3216.
 */
void maxima_mx_set_lsx_port(void **runtime, const void **mx, unsigned short port);

/**
 * Start the LSX server used for game communication.
 */
uintptr_t maxima_mx_start_lsx(void **runtime, const void **mx);

/**
 * Consume pending LSX events.
 */
uintptr_t maxima_mx_consume_lsx_events(void **runtime,
                                       const void **mx,
                                       const char ***events_out,
                                       unsigned int **event_pids_out,
                                       unsigned int *event_count_out);

/**
 * Free LSX events retrieved from [maxima_mx_consume_lsx_events].
 */
void maxima_mx_free_lsx_events(char **events, unsigned int event_count);

/**
 * Launch a game with Maxima, providing an EA Offer ID.
 */
uintptr_t maxima_launch_game(void **runtime, const void **mx, const char *c_offer_id);

/**
 * Find an owned game's offer ID by its slug.
 */
uintptr_t maxima_find_owned_offer(void **runtime,
                                  const void **mx,
                                  const char *c_game_slug,
                                  const char **offer_id_out);

/**
 * Get the local user's display name.
 */
uintptr_t maxima_get_local_display_name(void **runtime,
                                        const void **mx,
                                        const char **display_name_out);

/**
 * Pull the application's window into the foreground.
 */
uintptr_t maxima_take_foreground_focus(void);

/**
 * Read the path for an EA game.
 */
uintptr_t maxima_read_game_path(const char *c_name, const char **c_out_path);
