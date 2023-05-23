module DynamicInvocation {
    sequence<string> Hobbies;

    struct Person {
        string firstName;
        string lastName;
        int age;
        Hobbies hobbies;
    }

    interface Greeter {
        string greet(string name);
        void greetNTimes(string name, int n);
        string greetPerson(Person person);
    };
}
