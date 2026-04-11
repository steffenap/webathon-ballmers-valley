create table if not exists groups
(
    id integer primary key not null,
    name text not null
);

create table if not exists users
(
    id integer primary key not null,
    name text not null,
    health integer not null
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

insert into groups values (0, 'inf339b');
insert into users values (0, 'jonas', 5);
insert into users values (1, 'omfj', 6);
insert into user_group values (0, 0);
insert into user_group values (1, 0);
insert into tasks values (0, 'do inf339b', null, null, 1000000, 0);
