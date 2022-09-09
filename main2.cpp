#include <stdio.h>
#include <stdlib.h>
#include "NETMSG.pb.h"
#include <map>
#include <math.h>
#include "demofilebitbuf.cpp"




#define NET_MAX_PAYLOAD					( 262144 - 4 )		// largest message we can send in bytes
#define DEMO_RECORD_BUFFER_SIZE			( 2 * 1024 * 1024 )	// temp buffer big enough to fit both string tables and server classes

// How many bits to use to encode an edict.
#define	MAX_EDICT_BITS					11					// # of bits needed to represent max edicts
// Max # of edicts in a level
#define	MAX_EDICTS						( 1 << MAX_EDICT_BITS )

#define MAX_USERDATA_BITS				14
#define	MAX_USERDATA_SIZE				( 1 << MAX_USERDATA_BITS )
#define SUBSTRING_BITS					5

#define NUM_NETWORKED_EHANDLE_SERIAL_NUMBER_BITS	10

#define MAX_PLAYER_NAME_LENGTH			128
#define MAX_CUSTOM_FILES				4	// max 4 files
#define SIGNED_GUID_LEN					32	// Hashed CD Key (32 hex alphabetic chars + 0 terminator )

#define ENTITY_SENTINEL					9999

#define MAX_STRING_TABLES				64

struct StringTableData_t
{
	char	szName[ 64 ];
	int		nMaxEntries;
	int		nUserDataSize;      // not currently used to parse stringtable updates, kept for documentation purposes only
	int		nUserDataSizeBits;  // not currently used to parse stringtable updates, kept for documentation purposes only
	int		nUserDataFixedSize; // used to parse stringtable updates
};


struct ExcludeEntry
{
	ExcludeEntry( const char *pVarName, const char *pDTName, const char *pDTExcluding )
		: m_pVarName( pVarName )
		, m_pDTName( pDTName )
		, m_pDTExcluding( pDTExcluding )
	{
	}

	const char *m_pVarName;
	const char *m_pDTName;
	const char *m_pDTExcluding;
};

struct FlattenedPropEntry
{
	FlattenedPropEntry( const CSVCMsg_SendTable::sendprop_t *prop, const CSVCMsg_SendTable::sendprop_t *arrayElementProp )
		: m_prop( prop )
		, m_arrayElementProp( arrayElementProp )
	{
	}
	const CSVCMsg_SendTable::sendprop_t *m_prop;
	const CSVCMsg_SendTable::sendprop_t *m_arrayElementProp;
};

struct ServerClass_t
{
	int nClassID;
	char strName[256];
	char strDTName[256];
	int nDataTable;

	std::vector< FlattenedPropEntry > flattenedProps;
};

enum UpdateType
{
	EnterPVS = 0,	// Entity came back into pvs, create new entity if one doesn't exist
	LeavePVS,		// Entity left pvs
	DeltaEnt,		// There is a delta for this entity.
	PreserveEnt,	// Entity stays alive but no delta ( could be LOD, or just unchanged )
	Finished,		// finished parsing entities successfully
	Failed,			// parsing error occured while reading entities
};

// Flags for delta encoding header
enum HeaderFlags
{
	FHDR_ZERO			= 0x0000,
	FHDR_LEAVEPVS		= 0x0001,
	FHDR_DELETE			= 0x0002,
	FHDR_ENTERPVS		= 0x0004,
};


struct CommandHeader{
    unsigned char commandId;
    int currentTick;
    unsigned char playerId;
};

struct intAndPtr{
    int varint;
    FILE *ptr;
};

struct event{
    int type;
    std::string name;
};



struct EntityEntry
{
	EntityEntry( int nEntity, uint32 uClass, uint32 uSerialNum)
		: m_nEntity( nEntity )
		, m_uClass( uClass )
		, m_uSerialNum( uSerialNum )
	{
	}
	~EntityEntry()
	{
		for ( std::vector< PropEntry * >::iterator i = m_props.begin(); i != m_props.end(); i++ )
		{
			delete *i;
		}
	}
	PropEntry *FindProp( const char *pName )
	{
		for ( std::vector< PropEntry * >::iterator i = m_props.begin(); i != m_props.end(); i++ )
		{
			PropEntry *pProp = *i;
			if (  pProp->m_pFlattenedProp->m_prop->var_name().compare( pName ) == 0 )
			{
				return pProp;
			}
		}
		return NULL;
	}
	void AddOrUpdateProp( FlattenedPropEntry *pFlattenedProp, Prop_t *pPropValue )
	{
		//if ( m_uClass == 34 && pFlattenedProp->m_prop->var_name().compare( "m_vecOrigin" ) == 0 )
		//{
		//	printf("got vec origin!\n" );
		//}
		PropEntry *pProp = FindProp( pFlattenedProp->m_prop->var_name().c_str() );
		if ( pProp )
		{
			delete pProp->m_pPropValue;
			pProp->m_pPropValue = pPropValue;
		}
		else
		{
			pProp = new PropEntry( pFlattenedProp, pPropValue );
			m_props.push_back( pProp );
		}
	}
	int m_nEntity;
	uint32 m_uClass;
	uint32 m_uSerialNum;

	std::vector< PropEntry * > m_props;
};

#define MAX_STRING_TABLES				64

static std::vector< ServerClass_t > s_ServerClasses;

static int s_nServerClassBits = 0;
static std::vector< ServerClass_t > s_ServerClasses;
static std::vector< CSVCMsg_SendTable > s_DataTables;
static std::vector< ExcludeEntry > s_currentExcludes;
static std::vector< EntityEntry * > s_Entities;
//static std::vector< player_info_t > s_PlayerInfos;
EntityEntry *FindEntity( int nEntity );


int classbits;
static StringTableData_t s_StringTables[ MAX_STRING_TABLES ];
unsigned char data[100000];
int currentTick = 0;
CSVCMsg_GameEventList gel;
std::vector<int> hurtTicks;

std::map<int,std::string> packetNameMap {
    {0, "net_NOP"},
    {1, "net_Disconnect"},
    {2, "net_File"},
    {3, "net_SplitScreenUser"},
    {4, "net_Tick"},
    {5, "net_StringCmd"},
    {6, "net_SetConVar"},
    {7, "net_SignonState"},
    {100, "net_PlayerAvatarData"},
    {8, "svc_ServerInfo"},
    {9, "svc_SendTable"},
    {10, "svc_ClassInfo"},
    {11, "svc_SetPause"},
    {12, "svc_CreateStringTable"},
    {13, "svc_UpdateStringTable"},
    {14, "svc_VoiceInit"},
    {15, "svc_VoiceData"},
    {16, "svc_Print"},
    {17, "svc_Sounds"},
    {18, "svc_SetView"},
    {19, "svc_FixAngle"},
    {20, "svc_CrosshairAngle"},
    {21, "svc_BSPDecal"},
    {22, "svc_SplitScreen"},
    {23, "svc_UserMessage"},
    {24, "svc_EntityMessage"},
    {25, "svc_GameEvent"},
    {26, "svc_PacketEntities"},
    {27, "svc_TempEntities"},
    {28, "svc_Prefetch"},
    {29, "svc_Menu"},
    {30, "svc_GameEventList"},
    {31, "svc_GetCvarValue"},
    {33, "svc_PaintmapData"},
    {34, "svc_CmdKeyValues"},
    {35, "svc_EncryptedData"},
    {36, "svc_HltvReplay"},
    {38, "svc_Broadcast_Command"}
    };







inline int parseVarInt(FILE **ptr){
    int count, result;
    int cont = 1;
    unsigned char b;

    b = 0;
    count = 0;
    result = 0;
    cont = 1;
    while (cont != 0){
        fread(&b, sizeof(b), 1, *ptr);

        if (count < 5){
            result |= (b & 0x7F) << (7 * count);
        }
        count += 1;
        cont = b & 0x80;
    }
    return result;
}

inline int varint_size(int value){
    if (value < (1 << 7)){
        return 1;
    }
    else if (value < (1 << 14)){
        return 2;
    }
    else if (value < (1 << 21)){
        return 3;
    }
    else if (value < (1 << 28)){
        return 4;
    }
    else{
        return 5;
    }
}

class StringTable{
    public:
        StringTable(CSVCMsg_CreateStringTable msg){
            std::string name = msg.name();
            int maxEntries = msg.max_entries();
            int udfs = msg.user_data_fixed_size();
            int uds = msg.user_data_size();
            int udsb = msg.user_data_size_bits();
        };
};


void updateStringTable(const void* data, int size){
            CSVCMsg_UpdateStringTable msg;
            msg.ParseFromArray(data, size);
            CBitRead buf( &msg.string_data()[ 0 ], msg.string_data().size() );
            // printf( "UpdateStringTable:%d(%s):%d:\n", msg.table_id(), table.szName, msg.num_changed_entries() );
};






int ReadFieldIndex( CBitRead &entityBitBuffer, int lastIndex, bool bNewWay )
{
	if (bNewWay)
	{
		if (entityBitBuffer.ReadOneBit())
		{
			return lastIndex + 1;
		}
	}
 
	int ret = 0;
	if (bNewWay && entityBitBuffer.ReadOneBit())
	{
		ret = entityBitBuffer.ReadUBitLong(3);  // read 3 bits
	}
	else
	{
		ret = entityBitBuffer.ReadUBitLong(7); // read 7 bits
		switch( ret & ( 32 | 64 ) )
		{
			case 32:
				ret = ( ret &~96 ) | ( entityBitBuffer.ReadUBitLong( 2 ) << 5 );
				assert( ret >= 32);
				break;
			case 64:
				ret = ( ret &~96 ) | ( entityBitBuffer.ReadUBitLong( 4 ) << 5 );
				assert( ret >= 128);
				break;
			case 96:
				ret = ( ret &~96 ) | ( entityBitBuffer.ReadUBitLong( 7 ) << 5 );
				assert( ret >= 512);
				break;
		}
	}
 
	if (ret == 0xFFF) // end marker is 4095 for cs:go
	{
		return -1;
	}
 
	return lastIndex + 1 + ret;
}





/*
EntityEntry *AddEntity( int nEntity, uint32_t uClass, uint32_t uSerialNum )
{
	// if entity already exists, then replace it, else add it
	EntityEntry *pEntity = FindEntity( nEntity );
	if ( pEntity )
	{
		pEntity->m_uClass = uClass;
		pEntity->m_uSerialNum = uSerialNum;
	}
	else
	{
		pEntity = new EntityEntry( nEntity, uClass, uSerialNum );
		s_Entities.push_back( pEntity );
	}

	return pEntity;
}


EntityEntry *FindEntity( int nEntity )
{
	for ( std::vector< EntityEntry * >::iterator i = s_Entities.begin(); i != s_Entities.end(); i++ )
	{
		if (  (*i)->m_nEntity == nEntity )
		{
			return *i;
		}
	}

	return NULL;
}*/

void handleEntityUpdate(CBitRead buf){
    bool bNewWay = ( buf.ReadOneBit() == 1 );  // 0 = old way, 1 = new way

	std::vector< int > fieldIndices;

	int index = -1;
	do
	{
		index = ReadFieldIndex( buf, index, bNewWay );
		if ( index != -1 )
		{
			fieldIndices.push_back( index );
		}
	} while (index != -1);

    CSVCMsg_SendTable *pTable = GetTableByClassID( pEntity->m_uClass );
	if ( g_bDumpPacketEntities )
	{
		printf( "Table: %s\n", pTable->net_table_name().c_str() );
	}
	for ( unsigned int i = 0; i < fieldIndices.size(); i++ )
	{
		FlattenedPropEntry *pSendProp = GetSendPropByIndex( pEntity->m_uClass, fieldIndices[ i ] );
		if ( pSendProp )
		{
			Prop_t *pProp = DecodeProp( entityBitBuffer, pSendProp, pEntity->m_uClass, fieldIndices[ i ], !g_bDumpPacketEntities );
			pEntity->AddOrUpdateProp( pSendProp, pProp );
		}
		else
		{
			return false;
		}
	}

	return true;
}

CSVCMsg_SendTable *GetTableByClassID( uint32_t nClassID )
{
	for ( uint32_t i = 0; i < s_ServerClasses.size(); i++ )
	{
		if ( s_ServerClasses[ i ].nClassID == nClassID )
		{
			return &(s_DataTables[ s_ServerClasses[i].nDataTable ]);
		}
	}
	return NULL;
}

void readNewEntity(CBitRead buf, uint32_t uClass){
    ;//buf.inde
}


void parseEntities(FILE **ptr, int size, unsigned char* data){
    CSVCMsg_PacketEntities msg;
    msg.ParseFromArray(data, size);

    CBitRead buf( &msg.entity_data()[ 0 ], msg.entity_data().size() );
    int entityid = -1;
    for(int i = 0; i < msg.max_entries(); i++){
        bool bNewWay = ( buf.ReadOneBit() == 1 );  // 0 = old way, 1 = new way
        std::cout << bNewWay << "\n";
        if (bNewWay){
            buf.ReadOneBit();
        }else if ( buf.ReadOneBit() == 1 ){
            buf.ReadOneBit();
            uint32_t uClass = buf.ReadUBitLong( classbits );
            uint32_t uSerialNum = buf.ReadUBitLong( NUM_NETWORKED_EHANDLE_SERIAL_NUMBER_BITS );
            readNewEntity(buf, uClass);
            
            std::cout << "uclass" << uClass << " serialnum" << uSerialNum << "\n";
        }else{
            ;//readNewEntity(buf, uClass);
        }
    }
}



void createStringTabl(FILE **ptr, int size, unsigned char* data){
    CSVCMsg_CreateStringTable msg;
    msg.ParseFromArray(data, size);
    CBitRead buf( &msg.string_data()[ 0 ], msg.string_data().size() );
}


FILE* parsePacket(FILE *ptr){
    

    struct intAndPtr intaptr;
    int length;
    int size;

    fseek(ptr, 160, SEEK_CUR);
    fread(&length, sizeof(length), 1, ptr);
    int index = 0;

    while (index < length){
        
        unsigned char* buffer;
        
        int msg = parseVarInt(&ptr);
        int size = parseVarInt(&ptr);
        
        fread(&data, size, 1, ptr);

        if (msg == 12)
        {
            CSVCMsg_CreateStringTable msgstr;
            msgstr.ParseFromArray(data, size);
            //printf("CreateStringTable:%s:%d:%d:%d:%d:\n", msgstr.name().c_str(), msgstr.max_entries(), msgstr.num_entries(), msgstr.user_data_size(), msgstr.user_data_size_bits() );

        }

        if (msg == 26){
            ;
            //std::cout << "26\n";
            //parseEntities(&ptr, size, data);
        }

        if (msg == 25){
            CSVCMsg_GameEvent ge;
            ge.ParseFromArray(data, size);

            int numKeys = ge.keys().size();
            int idesc = 0;
            for (idesc = 0; idesc < gel.descriptors().size(); idesc++){
                const CSVCMsg_GameEventList::descriptor_t& Descriptor = gel.descriptors( idesc );
                if (Descriptor.eventid() == ge.eventid()){
                    //std::cout << "FOUND" << "\n";
                    break;
                }
            }
            // const CSVCMsg_GameEventList::descriptor_t& Descriptor = Demo.m_GameEventList.descriptors( iDescriptor );
            if (idesc == gel.descriptors().size()-1){
                    continue;
            }
        
            const CSVCMsg_GameEventList::descriptor_t& pDescriptor = gel.descriptors( idesc );
            
            //std::cout << pDescriptor.name().c_str() <<" <--------------- EVENT NAME\n";
            //std::cout << idesc << " " << pDescriptor.name().c_str() <<"\n";
            if (idesc == 25){
                hurtTicks.push_back(currentTick);
                // std::cout << currentTick << "<- Tick \n";
                for(int i = 0; i < numKeys; i++){

                    // std::cout << "PRE" << "\n";
                    
                    const CSVCMsg_GameEventList::key_t& Key = pDescriptor.keys( i );
                    const CSVCMsg_GameEvent::key_t& KeyValue =  ge.keys(i);
                    // std::cout << KeyValue.DebugString() << "\n";

                    // std::cout << ev.DebugString() << "\n";
                    // std::cout << pDescriptor.keys(i).name() << "<------ \n";
                    // std::cout << pDescriptor.keys(i).type() << "<------ TYPE\n";
                    
                    int typed = pDescriptor.keys(i).type();
                    
                    //printf(" %s: ", Key.name().c_str() );

                    if (typed == 1){
                        std::string strVal = KeyValue.val_string();
                        //std::cout << strVal << "\n";
                    }
                    else if(typed == 2){
                        float key_val_float = KeyValue.val_float();
                        //std::cout << key_val_float << "\n";
                    }
                    else if(typed == 3){
                        long longVal = KeyValue.val_long();
                        //std::cout << longVal << "\n";
                    }
                    else if(typed == 4){
                        short shortVal = KeyValue.val_short();
                        //std::cout << shortVal << "\n";
                    }
                    else if(typed == 5){
                        unsigned char byteVal = KeyValue.val_byte();
                        //std::cout << byteVal << "\n";
                    }
                    else if(typed == 6){
                        int boolVal = KeyValue.val_bool();
                        //std::cout << boolVal << "\n";
                    }
                    else if(typed == 7){
                        uint64_t u64Val = KeyValue.val_uint64();
                        //std::cout << u64Val << "\n";
                    }
                    else if(typed == 8){
                        ;//key_val = KeyValue.val_wstring();
                    }
                }
            }
        }
        if (msg == 30){
            gel.ParseFromArray(data, size);
        }
        index += varint_size(msg) + varint_size(size) + size;
        //free(buffer);
    }
    return ptr;
}

void parseConsoleCommand(FILE *ptr){
    ;
}

FILE* parseDataTables(FILE *ptr){
    printf("BENU\n");
    int v_type, size;
    int length = 0;
    CSVCMsg_SendTable msg;    
    
    length = fread(&length, sizeof(length), 1, ptr);
    
    //std::cout << length << " LENGTH\n";
    int rounds = 0;
    while(1)
    {
        rounds += 1;
        int v_type = parseVarInt(&ptr);

        // SIZE
        int size = parseVarInt(&ptr);


        unsigned char buffer[size];
        fread(&buffer, 1, size, ptr);
        msg.ParseFromArray(&buffer, size);
        
        if (msg.is_end())
        {
            break;
        }
    }

    short svClasses;
    fread(&svClasses, sizeof(svClasses), 1, ptr);

    int classbits = int(ceil(log2(svClasses)));

    short int tempId;
    unsigned char c;
    int cnt = 0;
    short playerID;
    

    for (int i = 0; i<int(svClasses); i++){
        std::string name;
        std::string dt;
        fread(&playerID, sizeof(playerID), 1, ptr);       
        // NAME
        while(1){
            fread(&c, sizeof(c), 1, ptr);
            if (c == '\0'){
                break;
            }
            name.push_back(c);
        }
        // DT
        while(1){
            fread(&c, sizeof(c), 1, ptr);
            if (c == '\0'){
                break;
            }
            dt.push_back(c);
        }

        std::cout << name << "  " <<dt << " <-- NAME AND DT \n";

    }
    return ptr;
}

struct ServerClass{
    short id;
    std::string name;
    std::string dt;
};


int main(){
    

    FILE *ptr;
    ptr = fopen("y.dem","rb");  // r for read, b for binary

    unsigned char header[1072];
    
    struct CommandHeader ch;
    struct intAndPtr intaptr;
    

    unsigned char commandId;
    
    unsigned char playerId;


    fread(header, sizeof(header), 1, ptr);
    int cnt = 0;
    while (1){
        cnt += 1;
        // CommandHeader

        fread(&commandId, sizeof(commandId), 1, ptr);
        fread(&currentTick, sizeof(currentTick), 1, ptr);
        fread(&playerId, sizeof(playerId), 1, ptr);
        // printf("Command ID: %u  Current tick: %d Player id:%u \n", commandId, currentTick, playerId);
        int cid;
        cid = commandId;
        
        //std::cout << currentTick << "\n";

        if (cid == 1 || cid == 2)
        {
            ptr = parsePacket(ptr);
        }
        
        else if (cid == 6)
        {
            ptr = parseDataTables(ptr);
        }
        else if (cid == 7)
        {
            break;
        }
    }
}