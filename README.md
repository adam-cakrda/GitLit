# GitLit
![logo](https://raw.githubusercontent.com/adam-cakrda/GitLit/refs/heads/master/public/gitlit.svg)

A simple github alternative written in rust

![last-commit](https://img.shields.io/github/last-commit/adam-cakrda/GitLit?style=flat&logo=git&logoColor=white&color=0080ff)
![GitHub commit activity](https://img.shields.io/github/commit-activity/t/adam-cakrda/GitLit)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/adam-cakrda/GitLit/.github%2Fworkflows%2Frust.yml)
![repo-top-language](https://img.shields.io/github/languages/top/adam-cakrda/GitLit?style=flat&color=0080ff)

---

## Table of Contents
- [Overview](#GitLit)
- [Getting Started](#getting-started)
    - [Usage](#usage)
    - [TODO](#todo)

---

## Security and Deployment

GitLit can be run on a public server. Private repositories are stored under the repos directory on disk and are not served as static files. Git over HTTP requests are handled by git_http_backend with an authentication layer (Basic auth for Git operations). Access to a private repository over HTTP is denied unless the request is authenticated. Public repositories can be read anonymously; private repositories require authentication.

Important recommendations for running on a public server:
- Do not expose your database to the internet. Bind MongoDB to localhost or a private network and use a firewall.
- Always run behind HTTPS (e.g., via a reverse proxy like Nginx, Caddy, or Traefik). Basic auth must be protected by TLS.
- Keep the repos directory accessible only to the GitLit process user. Do not mount it via a static file server or CDN.
- Keep .env/my.env and other secrets out of version control and out of your web root. They are now ignored in .gitignore by default.
- Use strong, unique passwords. Consider enabling additional factors when implemented.
- Validate that your PORT/address binding matches your threat model (the default binds to localhost; expose explicitly via a reverse proxy).
- Regularly update dependencies and rotate tokens/passwords.

If these practices are followed, storing private repositories on a public server is supported by GitLit’s current design.

## Getting Started

### Usage
Build GitLit from the source and install dependencies:

1. **Clone the repository:**
   ```sh
   git clone https://github.com/adam-cakrda/GitLit.git
   ```

2. **Navigate to the project directory:**
   ```sh
   cd GitLit
   ```

3. **Install the dependencies:**
   ```sh
   cargo build
   ```

4. **Run the project:**
   ```sh
   cargo run
   ```

### TODO

- [ ] Database
    - [x] MongoDB
    - [ ] Mysql? 
  
- [ ] Auth
    - [x] Register
    - [x] Login
    - [x] Logout
    - [ ] 2fa
        - [ ] Email
        - [ ] Authenticator
    - [x] Token
    - [ ] Git
        - [x] Basic auth
        - [ ] 2fa
    - [ ] Remember me
    - [ ] Change data
        - [ ] Password
        - [ ] Email
        - [ ] Username

- [ ] Application
    - [ ] Desktop 
        - [ ] Windows
        - [ ] Linux
        - [ ] MacOS
    - [ ] Mobile
        - [ ] Android
        - [ ] IOS
      
- [ ] Api
    - [x] Documentation - [gitlit.rostiapp.cz/api/docs/](https://gitlit.rostiapp.cz/api/docs/)
    - [x] v1
        - [x] login
        - [x] register
        - [x] logout
        - [x] create
        - [x] delete
        - [x] repos - get repos by filter
        - [x] branches
          - [x] Show
          - [ ] Delete
        - [x] commits
        - [x] content - of file or folder
        - [x] download as zip
    - [ ] v2
        - [ ] login
          - [ ] 2fa
          - [ ] remember me
        - [ ] register
            - [ ] 2fa
        - [ ] search
        - [ ] issues
        - [ ] pull requests
        - [ ] and more ....
      
- [ ] Frontend
    - [x] Login
    - [x] Register
    - [x] Logout
    - [ ] Error pages
    - [ ] Search
    - [ ] Profile
        - [x] Repositories
        - [ ] Picture
        - [ ] Activity
        - [ ] and more ....
    - [x] Home
    - [x] Repo
      - [x] Create
      - [ ] Delete
      - [x] Branches
        - [x] Show
        - [ ] Delete
      - [x] Commits
      - [x] Content
      - [ ] Issues
      - [ ] Pull requests
      - [ ] and more ....
  
- [ ] Repo
    - [x] Create
    - [x] Delete
    - [x] Branches
      - [x] Show
      - [ ] Delete
    - [x] Commits
    - [x] Content
    - [ ] Issues
    - [ ] Pull requests
    - [ ] and more ....

---

