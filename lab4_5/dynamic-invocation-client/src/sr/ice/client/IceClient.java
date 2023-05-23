package sr.ice.client;
import com.zeroc.Ice.*;
import com.zeroc.Ice.Exception;



public class IceClient {
    public static void main(String[] args) {
        int status = 0;
        Communicator communicator = null;
        try {
            communicator = Util.initialize(args);
            ObjectPrx proxy = communicator.stringToProxy("Greeter:tcp -h localhost -p 10000");

            OutputStream out1 = new OutputStream(communicator);
            out1.startEncapsulation();
            String name = "John";
            out1.writeString(name);
            out1.endEncapsulation();
            byte[] inParams = out1.finished();

            com.zeroc.Ice.Object.Ice_invokeResult r = proxy.ice_invoke("greet", OperationMode.Normal, inParams);
            if(r.returnValue)
            {
                com.zeroc.Ice.InputStream in = new InputStream(communicator, r.outParams);
                in.startEncapsulation();
                String result = in.readString();
                in.endEncapsulation();
                System.out.println(result);
            }

            OutputStream out2 = new OutputStream(communicator);
            out2.startEncapsulation();
            out2.writeString(name);
            int n = 3;
            out2.writeInt(n);
            out2.endEncapsulation();
            inParams = out2.finished();
            proxy.ice_invoke("greetNTimes", OperationMode.Normal, inParams);

            String[] hobbies = {"football", "gym", "books"};
            Person person = new Person("John", "Travolta", 33, hobbies);

            OutputStream out3 = new OutputStream(communicator);
            out3.startEncapsulation();
            out3.writeString(person.firstName);
            out3.writeString(person.lastName);
            out3.writeInt(person.age);
            out3.writeStringSeq(person.hobbies);
            out3.endEncapsulation();
            inParams = out3.finished();

            r = proxy.ice_invoke("greetPerson", OperationMode.Normal, inParams);
            if(r.returnValue)
            {
                com.zeroc.Ice.InputStream in = new InputStream(communicator, r.outParams);
                in.startEncapsulation();
                String result = in.readString();
                in.endEncapsulation();
                System.out.println(result);
            }

        } catch (Exception e) {
            e.printStackTrace();
            status = 1;
        } finally {
            if (communicator != null) {
                try {
                    communicator.destroy();
                } catch (Exception e) {
                    e.printStackTrace();
                    status = 1;
                }
            }
        }
        System.exit(status);
    }
}
