import Ice
import DynamicInvocation


class GreeterI(DynamicInvocation.Greeter):
    def greet(self, hello, current=None):
        print(f"Received greeting: {hello}")
        return f"Hello, {hello}!"

    def greetNTimes(self, name, n, current=None):
        print(f"Greeting {name} {n} times")
        for i in range(n):
            print(f"Greeting {i+1}: Hello, {name}!")

    def greetPerson(self, person, current=None):
        return f"Hi {person.firstName} {person.lastName}, you are {person.age} and intersted in {person.hobbies}. Nice to meet you!"


if __name__ == "__main__":
    with Ice.initialize() as communicator:
        adapter = communicator.createObjectAdapterWithEndpoints(
            "GreeterAdapter", "tcp -h 0.0.0.0 -p 10000"
        )
        adapter.add(GreeterI(), communicator.stringToIdentity("Greeter"))
        adapter.activate()
        communicator.waitForShutdown()
