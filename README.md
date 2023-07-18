# Dejavu

> The content in README.md is assistanted by ChatGPT.

## Overview

Dejavu is an open-source, cross-platform tool designed to help you record and search anything you've seen. With Dejavu, you can have a perfect memory by capturing and organizing your visual recordings efficiently. Whether you want to recall a decision from a meeting or locate that tweet you saw, Dejavu has got you covered.

### Features

- **Record and Store:** Dejavu allows you to effortlessly record and store visual recordings. Your data saved on your local machine, ensuring complete privacy and accessibility.

- **Search and Retrieval**: With Dejavu, you can quickly search and retrieve any specific moment or visual information you previously recorded, reducing the need for extensive note-taking. Easily find that important detail or revisit a particular captured image or video.

- **Cross-Platform Compatibility**: Dejavu is built to be cross-platform, supporting major operating systems such as Linux, Windows, and macOS. Enjoy the seamless experience and powerful features on your preferred device.

- **(TBD)Customizable Settings**: Tailor Dejavu to your needs by customizing various settings. Exclude specific applications from recording for enhanced privacy. Dejavu puts you in full control of your recording preferences.

## Getting Started

To start using Dejavu, follow these steps:

1. Clone the Repository: Begin by cloning the Dejavu repository to your local machine using the following command:

```bash
git clone https://github.com/strrl/dejavu.git
```

2. Build and Run: Build the Dejavu application and execute it on your machine. Refer to the documentation for specific build and execution instructions compatible with your operating system.

```bash
make
```

```bash
RUST_BACKTRACE=1 RUST_LOG=trace ./target/release/dejavu
```

3. Explore and Utilize: There is a simple webui embbed in dejavu: `http://localhost:12333`. Once Dejavu is running, start exploring its features. Record and store your desired visual moments, search and retrieve previous recordings, and customize the settings according to your preferences.

## Contributing

Contributions to Dejavu are more than welcome! If you'd like to contribute, please follow our [contribution guidelines](https://github.com/STRRL/dejavu/blob/master/CONTRIBUTING.md). We appreciate your help in making Dejavu even better. Dejavu require rust amd pnpm for development.

## License

Dejavu is released under the [MIT License](https://github.com/STRRL/dejavu/blob/master/LICENSE). Feel free to use, modify, and distribute the tool in compliance with the terms of the license.

## Support and Feedback

For any questions, issues, or feedback, please open an issue on the Dejavu repository. Our team will be glad to assist you.

Thank you for choosing Dejavu! We hope it becomes your go-to tool for capturing and recalling important visual moments in your life.
