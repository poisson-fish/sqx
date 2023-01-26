<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a name="readme-top"></a>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/udidifier/sqx">
    <img src="images/logo.png" alt="Logo" width="200" height="100">
  </a>

  <h3 align="center"></h3>

  <p align="center">
    A <a href="https://surrealdb.com/"><strong>SurrealDB</strong></a> powered data format swiss army knife.
    <br />
    <a href="https://surrealdb.com/docs/surrealql"><strong>Explore the SurrealQL docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/poisson-fish/sqx/issues">Report Bug</a>
    ·
    <a href="https://github.com/poisson-fish/sqx/issues">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about">About</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#build">Build</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About

<!-- [![Product Name Screen Shot][product-screenshot]](https://example.com) -->

SQX is a WIP tool to query, aggregate, filter, and convert structured data formats on disk. Leveraging <a href="https://surrealdb.com/"><strong>SurrealDB's</strong></a> flexible in memory database gives SQX a 

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

[![Rust][Rust]][rust-url] [![SurrealDB-Badge][SurrealDB-Badge]][surrealdb-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

Eventually I'll setup CI but for now you must build from source.

### Prerequisites

You'll need git, cargo and rustup.

https://rustup.rs/

### Build


1. Clone the repo
   ```sh
   git clone https://github.com/poisson-fish/sqx.git
   cd sqx
   ```
2. Build release
   ```sh
   cargo build --release
   ```
3. Copy sqx binary to your preferred runpath.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

(WIP)
* SQX is in infant stages and functionality is limited (for now!!)
* SQX can ingest only JSON via stdin or file blob for now, more formats to come (CSV, TSV, Apache Arrow via Pola.rs) .
* SQX only outputs in visual table format for now, both input and output format flags do nothing. 
* SQX takes quoted SurrealQL queries with the -s flag.

To query on stdin use the stdin table:
```sh
ps auxw | jc --ps | ./sqx -s "SELECT command,mem_percent FROM stdin ORDER BY mem_percent DESC LIMIT 10;"
```

To query on file input you use the 'filein' table.
Specify a space separated list of file blobs with a options postfixed double dash:
```sh
./sqx -vv -s "SELECT * FROM filein;" -- testdata/* 
```

_For more specific and powerful query language examples, please refer to the [SurrealQL Documentation](https://surrealdb.com/docs/surrealql)_

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ROADMAP -->
## Roadmap
- [x] Stdin input
- [x] Blob filepath input
- [ ] Multiformat ingest support
    - [x] JSON
    - [ ] TSV
    - [ ] CSV
    - [ ] Arrow
    - [ ] Raw posix tool output parsers
- [ ] Multiformat export support
    - [x] JSON
    - [ ] TSV
    - [ ] CSV
    - [ ] Arrow
    - [ ] SQL
    - [x] Tabled
- [-] Tables
    - [x] Basic table view
    - [ ] Optional
- [ ] Tests


See the [open issues](https://github.com/poisson-fish/sqx/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

twin - hyperviridian@gmail.com

Project Link: [https://github.com/poisson-fish/sqx](https://github.com/poisson-fish/sqx)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

Big credit to SurrealDB for building the next generation database in Rust!

![SurrealDB](https://raw.githubusercontent.com/surrealdb/surrealdb/main/img/logo.svg)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[SurrealDB-Badge]: https://img.shields.io/badge/SurrealDB-FF00A0?logo=surrealdb&logoColor=fff&style=for-the-badge
[surrealdb-url]: https://surrealdb.com/
[Rust]: https://img.shields.io/badge/rust-B94700?style=for-the-badge&logo=rust&logoColor=white
[rust-url]: https://www.rust-lang.org/