# Bluesky Watcher Bot - Post Notification Watcher

Follow me on Bluesky!
- [return Ok(elaina);](https://bsky.app/profile/elynn.bsky.social)

## Summary

Here's a summary of what this README.md covers. I tried to be as comprehensive as possible, to help both end users and anyone that wants to self-host this.

#### For End Users:

The only relevant section is probably [2.1 How to Use](#21-how-to-use). So you might as well just skip to that.

#### For maintainers, self-hosters, contributors:

I recommend reading the entire readme, or whatever sections below might help you. It covers all of the important aspects, really. The code is also properly documented, and I tried to make the variables self-explanatory for clarity, too.

**Table of Contents**

- [1. Special Thanks](#1-special-thanks)
- [2. Overall Description](#2-overall-description)
  - [2.1 How to Use](#21-how-to-use)
- [3. Current Features](#3-current-features)
  - [3.1 Error Handling](#31-error-handling)
  - [3.2 TODOs](#32-todos)
- [4. Workspace Organization](#4-workspace-organization)
- [5. Running and Building](#5-running-and-building)
  - [5.1 Makefile](#51-makefile)
  - [5.2 Environment Variables](#52-environment-variables)
  - [5.3 Production Deployment](#53-production-deployment)
  - [5.4 Docker Deployment (untested)](#54-docker-deployment-untested)
- [6. Contributions](#6-contributions)
- [7. FAQ / Troubleshooting](#7-faq--troubleshooting)
- [8. Licensing](#8-licensing)

---

### 1. Special Thanks

Special thanks to [Yoshihiro Sugi](https://github.com/sugyan), the author of **ATrium**, a collection of Rust libraries designed to work with the AT Protocol. ATrium successfully provided a versatile and coherent ecosystem that made the development of this bot possible and smooth.

Deep appreciation for the dedication and continuous development of ATrium, and I am grateful for the ongoing improvements.
Yoshihiro-san also was quick to help me when I had issues with it, as you can see [in this closed issue](https://github.com/sugyan/atrium/issues/220).

---

### 2. Overall Description

This **Bluesky Bot**, named **Watcher**, is designed to subscribe to post notifications from users on Bluesky and notify listeners in real-time. Built using [ATrium](https://github.com/sugyan/atrium), Watcher tracks posts and replies by interacting with ATProto. It is capable of monitoring multiple users simultaneously and employs [Tokio](https://tokio.rs/) to manage tasks and threads efficiently. It also includes a logging system to track all events and operations.

For the self-hosters and maintainers, there is a Discord webhook opt-in for the logging system. If you use it, it sends real-time updates to the webhook, keeping you informed of important events, including failures and changes to the watchlist. [The level of the logs can be configured with an environment variable](#52-environment-variables).

The bot is designed with robust error resilience, including a retry mechanism for API failures, connection issues, and invalid user inputs. This ensures continuous operation even in the face of temporary disruptions.

### 2.1 How to Use

The Watcher is operated directly through the Bluesky platform. Users can interact with it by sending specific commands via direct message (DM) to manage which users they want to watch for post notifications.

#### Available Commands:

- `!watch @user_1.handle @user_2.handle (...)`: Add one or more users to your watchlist. The bot will notify you whenever these users post or reply to posts.
   - NOTICE: Currently, listening to replies is not really implemented, but the code is structured to allow for such feature. If you really want this, [feel free to contribute](#6-contributions)!
  
- `!unwatch @user_1.handle @user_2.handle (...)`: Remove one or more users from your watchlist, stopping notifications for their posts or replies.

- `!list_watched`: View a list of all users you are currently watching.

- `!help`: Displays the available commands and their usage.

Make sure you follow the bot or at least have DMs opened for everyone, or else it won't be able to contact you!

#### **Opting Out**

Respect for user privacy and consent is a core guideline for this. If you wish to opt out of notifications, you can simply block the bot on Bluesky. This action will prevent it from sending you any notifications, as well as watching you. Your decision will be respected immediately.

---

### 3. Current Features

The Watcher comes with a variety of features designed to provide efficient and reliable notifications for users. 

#### **Key Features:**

- **Post Notifications**: Subscribes to posts and replies from specified users and sends real-time updates to listeners.

- **Session Caching**: Caches sessions to reduce repeated authentication.

- **In-Memory Repository**: Implements an in-memory repository for fast concurrent access to the watchlist and notifications.

- **Sqlite Storage**: Utilizes Sqlite to cache the state, ensuring persistence across restarts. This allows the bot to recover its state and resume operations without losing data.

- **Logging System**: Tracks all significant events and operations, providing detailed logs for monitoring and debugging.

- **Discord Webhooks**: Optionally, integrates with Discord to notify a channel about updates, ensuring immediate awareness of important logs (errors, warnings).

- **Graceful Shutdown**: Ensures that the Sqlite database disconnects gracefully before shutdown, maintaining data integrity and preventing corruption. Also finishes sending all your Discord logs, if the feature is enabled.

### 3.1 Error Handling

The Watcher is designed to handle a range of errors and potential issues gracefully, ensuring minimal disruption to its functionality.

#### **API and Bluesky Errors**

- **Retry Mechanism**: Attempts to issue requests and handle errors by retrying up to [`PER_REQ_MAX_RETRIES`](https://github.com/oestradiol/bsky-post-notifs-bot/blob/main/src/other/bsky/lib.rs#L158). This helps prevent single failures from interrupting the workflow.

#### **ATrium Bugs**

- **Bug Handling**: Includes safeguards to manage potential bugs in the ATrium library. These are addressed as they occur.

#### **Database Errors**

- **Sqlite Robustness**: Sqlite is known for its robustness, so database errors are rare. However, any issues that do arise are handled with appropriate error logging.

#### **API Failures**

- If API failures occur, the bot will retry in incrementing intervals [(1s, up to `INCREMENTS_LIMIT`, for a maximum of `MINUTES_LIMIT`)](https://github.com/oestradiol/bsky-post-notifs-bot/blob/main/src/other/utils/handle_api_failure.rs#L6). This is to mitigate temporary issues and ensure continued operation.

#### **Bluesky Bugs**

- The bot is prepared to handle any possible Bluesky bugs. It should log those comprehensively, so that the hoster can ask for help and possibly [contribute to the project](https://github.com/bluesky-social/atproto).

#### **Other Errors**

- **Command Handling Failures**: Handles cached commands. Command failures are logged, but it does not notify the sender about the failure, as currently any command failure is caused by a failure in contacting the API, so it's contradictory to attempt to notify them anyways. This means that failed commands need to be reissued.

- **Command Listener Failures**: Listens for new commands and [fetches unread conversations periodically](https://github.com/oestradiol/bsky-post-notifs-bot/blob/main/src/other/services/jobs/command_listener.rs#L11). Failures are logged, and the job will cancel if the error is deemed unrecoverable. Persistent failures will cause it to stop listening for new commands.

- **Post Watching Failures**: Watching users' posts and notifying watchers is [done periodically](https://github.com/oestradiol/bsky-post-notifs-bot/blob/main/src/other/services/jobs/user_watcher.rs#L26). If failures occur, they are logged, and the job will cancel if the error is unrecoverable.
    * Ideally, this would be reworked to use the firehose (or jetstreams). Feel free to contribute!

#### **Panic Scenarios**

- **Signal Handlers**: Panics if signal handlers for SIGTERM/SIGINT fail to install. This is crucial for handling termination signals properly.

- **Environment Variables**: Panics if required environment variables are not found or fail to parse correctly. Ensure all necessary variables are defined and correctly formatted.

- **Database Initialization**: Panics if the connection pool fails to initialize. This occurs if there are issues with creating or connecting to the database file.

- **Logging System**: Panics if the logging system fails to initialize, usually due to issues with environment variable definitions for log directory and severity level.

**Note**: Most panics occur during the initial setup, often related to environment configuration issues. Once initialized, the bot is designed to handle errors robustly.

---

### 3.2 TODOs

The following features and improvements are planned for future development (if it does ever happen):

- **Analysing and implementing using the firehose/jetstreams**: It'd be better to do that instead of making individual requests every 15s to the API. However, ATrium doesn't yet have a client compatible with Event Streams, but there's a [PR made my me](https://github.com/sugyan/atrium/pull/228) to work on this.

- **`with_replies` feature**: [Develop the `posts_with_replies` filter](https://github.com/oestradiol/bsky-post-notifs-bot/blob/main/src/other/bsky/get_last_post_time.rs#L22) to distinguish between replies and regular posts. This will enhance notification management for users who also want to be notified for replies.

- **Rate Limiting**: Analyze how the ATProto APIs handle rate limiting and [implement a more robust solution](https://github.com/oestradiol/bsky-post-notifs-bot/blob/main/src/other/bsky/lib.rs#L183) to manage potential rate limits, if necessary.

- **Configuration for Invalid Messages and Unknown Commands**: Creating a configuration file for customizing the response message for invalid messages and unknown commands. Currently, the messages are hard-coded ([occurrence 1](https://github.com/oestradiol/bsky-post-notifs-bot/blob/main/src/other/services/commands/invalid.rs#L7), [occurrence 2](https://github.com/oestradiol/bsky-post-notifs-bot/blob/main/src/other/services/commands/unknown.rs#L7)).

---

### 4. Workspace Organization

The project workspace is organized as follows:

- **`src/app`**: Contains the main application logic and entry point for the bot.

- **`src/other/bsky`**: Includes modules specific to interactions with the Bluesky API.

- **`src/other/environment`**: Manages environment configuration and related utilities.

- **`src/other/repositories`**: Houses data repositories, including the Sqlite storage and in-memory cache.

- **`src/other/services`**: Contains various services used, such as command processing and notification handling.

- **`src/other/utils`**: Includes utility functions and helpers used across the project.

---

### 5. Running and Building

First, ensure you're using the correct version of Rust (1.81). Then, to run **Watcher**, follow the instructions below for different environments and setups.

#### 5.1 Makefile

The Makefile provides convenient commands for building, testing, and running. Available commands:

- **`make clean`**: Cleans up build artifacts and logs while keeping the `.env` file.
- **`make build`**: Builds and copies the executable and `.env` file to the `dist` directory.
- **`make lint`**: Runs the linter for the codebase.
- **`make lint-fix`**: Automatically fixes linting issues.
- **`make force-lint-fix`**: Forces linting fixes even for dirty or staged files.
- **`make dev`**: Runs in development mode with full span tracing and backtraces, while still respecting the `LOG_SEVERITY` environment variable.
- **`make prod`**: Builds and runs in production mode.

#### 5.2 Environment Variables

Watcher requires several environment variables to function correctly. Ensure at least the required ones (the ones that don't have a default value) are set in your `.env` file or your environment:

- **`LOG_SEVERITY`**: Defines the severity level for logging (defaults to `INFO`).
- **`LOG_DIRECTORY`**: Directory where log files are stored (defaults to `/var/log/post_watcher`).
- **`DATABASE_URL`**: URL for the database (defaults to `sqlite://data.db`).
- **`DB_CONN_POOL_MAX`**: Maximum number of database connections (defaults to `100`).
- **`DISCORD_WEBHOOK`**: The Discord Webhook URL (does not have a default value, however this feature will be disabled if undefined).
- **`BOT_USERNAME`**: The bot's username on Bluesky.
- **`BOT_PASSWORD`**: The bot's password or app password.
- **`TURN_OFF_WATCHED_NOTIFS`**: Setting this variable to anything will prevent the bot from sending notifications to a newly watched user that they are being watched. Will also not send notifications when the user is unwatched by all their watchers. The feature is on by default.

An example `.env` file is provided as `.env.example`.

#### 5.3 Production Deployment

For production deployment:

1. **Build Watcher**: Use `make build` to compile and prepare the executable.
2. **Run Watcher**: Navigate to the `dist` directory and execute `./app` to start.

Alternatively, you can run `make prod` to do both of the commands above at once.

After that, you can copy the `dist` directory wherever you prefer and delete the rest of the source code.

Ensure all environment variables are set correctly before running.

#### 5.4 Docker Deployment (untested)

NOTICE: This section and the corresponding Dockerfile are untested, but it _should_ hopefully work.

To deploy using Docker, follow these steps:

1. **Build the Docker Image**:

  ```bash
  docker build -t watch-bot .
  ```

2. **Run the Docker Container**:

  ```bash
  docker run -d --name watch-bot -e BOT_USERNAME=<your_username> -e BOT_PASSWORD=<your_password>
  ```

Make sure to replace `<your_username>` and `<your_password>` with your actual bot credentials.

---

### 6. Contributions

Contributions are welcome and encouraged! If you'd like to help enhance **Watcher**, please submit issues or pull requests on our GitHub repository. Your support is greatly appreciated!

---

### 7. FAQ / Troubleshooting

<details>
   <summary><b><i>What should I do if the bot is not sending notifications?</b></i></summary>
Check the logs for errors. Ensure it has the correct permissions and that the environment variables are properly set. Also make sure the receiver is following the bot, or has DMs opened, or else the bot cannot contact them.
</details>

<details>
   <summary><b><i>How can I check if it is properly connected to Bluesky?</b></i></summary>
Verify the authentication details and check the connection status in the logs. Make sure your credentials are correct.
</details>

<details>
   <summary><b><i>The bot crashed or stopped working. What should I do?</b></i></summary>
Review the logs for any critical errors or panics. Restart it and monitor for recurring issues. If the problem persists, consider reporting it on the GitHub repository.
</details>

<details>
   <summary><b><i>How can I update the bot to the latest version?</b></i></summary>
Pull the latest changes from the repository, rebuild it using `make build`, and redeploy it.
</details>

<details>
   <summary><b><i>The bot is not responding to commands, or receiving any at all. What could be wrong?</b></i></summary>
If you are an end user, contact the maintainer. If you are the maintainer, first, ensure that the bot account has DMs opened for anyone. You can do that in the settings of your Bluesky account. Then, ensure that the commands sent by the users are correctly formatted and that it is actively listening for new commands. Check the logs for any errors related to command processing.
</details>

<details>
   <summary><b><i>How do I opt out of notifications/being watched?</b></i></summary>
You can block it on Bluesky to opt out. The bot respects user privacy and will stop sending notifications and watching you if blocked.
</details>

For additional help, check the [GitHub issues](https://github.com/oestradiol/bsky-post-notifs-bot/issues) or make a new issue for support.

---

### 8. Licensing

This project is licensed under the **BSD 3-Clause New or Revised License**. 

- **Permissive Use**: Free to use, modify, and distribute.
- **Attribution**: Must retain copyright notice and disclaimers.
- **No Endorsement**: Cannot use project names or contributors' names for promotion without permission.
- **Patent Grant**: Includes an express patent grant.

For more details, see the [BSD 3-Clause License File](https://github.com/oestradiol/bsky-post-notifs-bot/blob/main/LICENSE).
