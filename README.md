# tis100pad

[![Stories in Ready](https://badge.waffle.io/pollend/tis100pad.svg?label=ready&title=Ready)](http://waffle.io/pollend/tis100pad)

A web site for uploading and sharing solutions for the game TIS-100. 

##Feature
###Current
* A user account system
* Anonmous user or registerd user can submit solution
* Registered user can submit problems
* Both solutions and problems can be reviewied by Anonmous/Registed users and resaved as seperate entries

###Planned
* Solution Validation

##Installing

###Required Libraries

system libraries

```
Redis
Python
```

npm libraries

```
npm install -g grunt-cli
npm install -g bower
```

pip libraries

```
pip install flask
pip install flask-sqlalchemy
pip install Flask-KVSession
pip install Redis
pip install requests
pip install lupa
pip install psutil
pip install alembic
```

###Install

####Configuration

under `src/server/app/config.py.sample` is the sample configuration file for the server

```
cp config.py.sample config.py
```

```
import os
_basedir = os.path.abspath(os.path.dirname(__file__))

DEBUG = True

IP = "192.168.0.14"

SQLALCHEMY_DATABASE_URI = 'sqlite:///' + os.path.join(_basedir, 'app.db')
DATABASE_CONNECT_OPTIONS = {}

THREADS_PER_PAGE = 8

CSRF_ENABLED = True

RECAPTCHA_PRIVATE_TOKEN = "qnbauwbefliuabwel"
RECAPTCHA_PUBLIC_TOKEN = "qnbauwbefliuabwel"

PASSWORD_HASH="auwnpanwe8fawefh05sfgs"

SECRET_KEY = "aps8fp9a8wnef8ansdfa"
PERMANENT_SESSION_LIFETIME = datetime.timedelta(hours = 1)

GOOGLE_ANALYTICS_CODE = ""
```

`SQLALCHEMY_DATABASE_URI` this is the configuration url for sqlAlchemy to connect to a database. The current configuration creates a local sqlite database.

NOTE: for `sqlite:\\`  migrating backwards with the current configuration might become problematic due to the way sqlite works. i.e removing columns isn't possible. scale is a limitation of sqlite.

more information can be found here: http://docs.sqlalchemy.org/en/latest/core/engines.html

`PASSWORD_HASH` and `SECRET_KEY` are used to prevent a brute force attack

`PERMANENT_SESSION_LIFETIME` determins how long a session lasts

`RECAPTCHA_PRIVATE_TOKEN` and `RECAPTCHA_PUBLIC_TOKEN` register a recaptcha from [here](http://www.google.com/recaptcha/intro/index.html) and provide both a public and private token

`GOOGLE_ANALYTICS_CODE` register a google analytic id from [here](http://www.google.com/analytics/)

`THREADS_PER_PAGE` determins how many threads are used

####Inital Install

```
npm install
bower install
grunt build
grunt migrate
```
run these four commands in the root directory of the project to install all the necessary npm modules, installing bowers source files, building src files and setup the database and migrate to the current version of the project.

between each pull from source `grunt migrate` is needed to upgrade the current database to reflect the current release

####Site Configuration

It's recommend to configure the site using [gunicorn](http://gunicorn.org/) using ngnix as a proxy.

more information about deployment can be found here: http://docs.gunicorn.org/en/latest/deploy.html

##Modifying

```
grunt wtch
```

grunt will watch the src directory for any changes and rebuild the the project accordingly. Flask will restart with any changes to to the base server files but may occasionally crash with syntax errors.

```
grunt clean-up-bin
```

This command will clear the bin directory

#Licence
MIT
