create table if not exists groups
(
    id integer primary key not null,
    name text not null,
    admin integer not null
);

create table if not exists users
(
    id integer primary key not null,
    username text unique not null,
    password text not null,
    full_name text not null,
    health integer not null,
    health_last_tick integer not null,
    death_count integer not null
);

create table if not exists user_group
(
    user integer not null,
    gr integer not null
);

create table if not exists tasks
(
    id integer primary key not null,
    title text not null,
    type text,
    due integer,
    reward integer not null,
    gr integer
);

create table if not exists done
(
    user integer not null,
    task integer not null,
    unique (user, task)
);

insert into groups values (0, 'inf339b', 0);
insert into users values (0, 'jonas', 'abc', 'Jonas Haukenes', 1000, 1775984178, 10);
insert into users values (1, 'omfj', 'abc', 'Omf J', 1000, 1775984178, 0);
insert into user_group values (0, 0);
insert into user_group values (1, 0);
insert into tasks values (0, 'do inf339b', null, null, 1000000, 0);
