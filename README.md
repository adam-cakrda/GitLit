# GitLit
![logo](https://raw.githubusercontent.com/adam-cakrda/GitLit/refs/heads/master/public/gitlit.svg)

A simple github alternative written in rust

> ⚠️ **Warning:** GitLit is still in **early development**.  
> Features may be incomplete, unstable, or subject to change at any time.

![last-commit](https://img.shields.io/gitlab/last-commit/adam-cakrda/GitLit?style=flat&logo=git&logoColor=white&color=0080ff)
![GitLab Top Language](https://img.shields.io/gitlab/languages/adam-cakrda%2FGitLit)
![GitLab License](https://img.shields.io/gitlab/license/adam-cakrda%2Fgitlit?color=green)
![pipeline status](https://gitlab.com/adam-cakrda/GitLit/badges/master/pipeline.svg)](https://gitlab.com/adam-cakrda/GitLit/-/commits/master)

---

## Table of Contents
- [Overview](#GitLit)
- [Getting Started](#getting-started)
    - [Usage](#usage)
    - [TODO](#todo)

---

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
    - [x] Mysql 
  
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
    - [x] CLI - [gitlab.com/adam-cakrda/GitLit-CLI](https://gitlab.com/adam-cakrda/GitLit-CLI)
    - [ ] Desktop - In work - [gitlab.com/adam-cakrda/GitLit-Desktop](https://gitlab.com/adam-cakrda/GitLit-Desktop)
    - [ ] Mobile
    - [ ] Server config  
      
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
          - [x] Delete
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
        - [x] Delete
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
      - [x] Delete
    - [x] Commits
    - [x] Content
    - [ ] Issues
    - [ ] Pull requests
    - [ ] and more ....

---



