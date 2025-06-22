# tis100pad

A web site for uploading and sharing solutions for the game TIS-100. 

## Feature
### Current

* A user account system
* Anonmous user or registerd user can submit solution
* Registered user can submit problems
* Both solutions and problems can be reviewied by Anonmous/Registed users and resaved as seperate entries

### Planned
* Solution Validation

## Set Up
```
sqlx migrate run
```

## Javascript/css

```
npm install
npm run dev:watch 
```


## Server

```
docker-compose up
```

access the local shell of the application
```
docker-compose exec tis100 /bin/bash 
```

# Licence
MIT
